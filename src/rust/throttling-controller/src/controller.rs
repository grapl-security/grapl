use std::time::{
    SystemTime,
    Duration,
};

use figment::{
    Figment,
    providers::Env,
};
use grapl_config::PostgresClient;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::db::{
    ControllerDb,
    self,
    ControllerDbError,
};

/// This implementation of a PID controller comes from
///
/// ```text
/// Åström, K. J., & Hägglund, T. (1988).
/// Automatic Tuning of PID Controllers.
/// Instrument Society of America (ISA).
/// ISBN 1-55617-081-5
/// ```
///
/// While I've used English natural language names for the public API, I've kept
/// the mathematical symbols (at least within reason) faithful to their typeset
/// representation. This code is meant to be read alongside a copy of the book!
/// I found that keeping these variable names closely matched to the mathematics
/// aids understanding by reducing the amount of mental indirection, making
/// implementation errors easier to spot.
///
/// If you have any questions about this code feel free to reach out:
///
/// ```text
/// Jesse C. Grillo
/// jgrillo@protonmail.com
/// ```
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
struct PidController {
    K: f64,
    T_i: f64,
    T_d: f64,
    T_t: f64,
    N: f64,
    b: f64,
    P_k1: f64,
    I_k1: f64,
    D_k1: f64,
    r_k1: f64,
    y_k1: f64,
    y_k2: f64,
    t_k1: SystemTime,
}

#[allow(non_snake_case)]
impl PidController {

    /// Construct a new PidController.
    /// \
    /// # Arguments:
    /// \
    /// - `proportional_gain` -- The output the controller is multiplied by this
    ///   factor
    /// \
    /// - `integral_time_constant` -- The time required for the integral term to
    ///   "catch up to" the proportional term in the face of an instantaneous
    ///   jump in controller error
    /// \
    /// - `derivative_time_constant` -- The time required for the proportional
    ///   term to "catch up to" the derivative term if the error starts at zero
    ///   and increases at a fixed rate
    /// \
    /// - `tracking_time_constant` -- The time required for the integral term to
    ///   "reset". This is necessary to prevent integral wind-up.
    /// \
    /// - `derivative_gain_limit` -- This term mitigates the effects of high
    ///   frequency noise in the derivative term by limiting the gain, which in
    ///   turn limits the high frequency noise amplification factor.
    /// \
    /// - `set_point_coefficient` -- This term determines how the controller
    ///   reacts to a change in the setpoint.
    pub fn new(
        proportional_gain: f64,
        integral_time_constant: f64,
        derivative_time_constant: f64,
        tracking_time_constant: f64,
        derivative_gain_limit: f64,
        set_point_coefficient: f64,
    ) -> Self {
        Self {
            K: proportional_gain,
            T_i: integral_time_constant,
            T_d: derivative_time_constant,
            T_t: tracking_time_constant,
            N: derivative_gain_limit,
            b: set_point_coefficient,
            P_k1: 0.0,
            I_k1: 0.0,
            D_k1: 0.0,
            r_k1: 0.0,
            y_k1: 0.0,
            y_k2: 0.0,
            t_k1: SystemTime::now(),
        }
    }

    // TODO: numerical stability
    fn calculate_P(&self, r_k: f64, y_k: f64) -> f64 {
        self.P_k1 + self.K * (self.b * r_k - y_k - self.b * self.r_k1 + self.y_k1)
    }

    // TODO: numerical stability
    fn calculate_I(&self, h: f64, r_k: f64, y_k: f64, u: f64, v: f64) -> f64 {
        self.I_k1 + (self.K * h / self.T_i) * (r_k - y_k) + (h / self.T_t) * (u - v)
    }

    // TODO: numerical stability
    fn calculate_D(&self, h: f64, y_k: f64) -> f64 {
        let (a_i, b_i) = if self.T_d < (self.N * h) / 2.0 {
            // use backward difference
            let a_i = self.T_d / (self.T_d + self.N * h);
            let b_i = -1.0 * self.K * self.T_d * self.N / (self.T_d + h * self.N);

            (a_i, b_i)
        } else {
            // use Tustin's approximation
            let a_i = (2.0 * self.T_d - h * self.N) / (2.0 * self.T_d + h * self.N);
            let b_i = -2.0 * self.K * self.N * self.T_d / (2.0 * self.T_d + h * self.N);

            (a_i, b_i)
        };

        self.D_k1 + (b_i / (1.0 - a_i)) * (y_k - 2.0 * self.y_k1 + self.y_k2)
    }

    fn update_state(
        &mut self,
        h: f64,
        r_k: f64,
        y_k: f64,
        u_low: f64,
        u_high: f64
    ) -> f64 {
        self.P_k1 = self.calculate_P(r_k, y_k);
        self.D_k1 = self.calculate_D(h, y_k);

        let v = self.control_output();
        let u = sat(v, u_low, u_high);

        self.I_k1 = self.calculate_I(h, r_k, y_k, u, v);

        self.r_k1 = r_k;
        self.y_k2 = self.y_k1;
        self.y_k1 = y_k;

        self.control_output()
    }

    /// Updates this controller's state according to the given
    /// parameters. Returns the updated control output.
    /// \
    /// # Arguments:
    /// \
    /// - `set_point` -- the desired value for the process variable
    /// \
    /// - `process_measurement` -- the measured process variable value
    /// \
    /// - `measurement_time` -- the time when the `process_measurement` was made
    /// \
    /// - `lower_saturation_limit` -- the minimum value the controller can take
    /// \
    /// - `upper_saturation_limit` -- the maximum value the controller can take
    pub fn update(
        &mut self,
        set_point: f64,
        process_measurement: f64,
        measurement_time: SystemTime,
        lower_saturation_limit: f64,
        upper_saturation_limit: f64
    ) -> f64 {
        let h = match measurement_time.duration_since(self.last_update_time()) {
            Ok(duration) => duration.as_secs_f64(),
            Err(e) => Duration::from_nanos(1).as_secs_f64(), // FIXME?
        };

        self.update_state(
            h,
            set_point,
            process_measurement,
            lower_saturation_limit,
            upper_saturation_limit
        )
    }

    /// Returns the most recently computed control output
    pub fn control_output(&self) -> f64 {
        self.P_k1 + self.I_k1 + self.D_k1
    }

    /// Returns the last time this controller was updated
    pub fn last_update_time(&self) -> SystemTime {
        self.t_k1
    }
}

impl From<db::Controller> for PidController {
    fn from(controller: db::Controller) -> Self {
        Self {
            K: controller.K(),
            T_i: controller.T_i(),
            T_d: controller.T_d(),
            T_t: controller.T_t(),
            N: controller.N(),
            b: controller.b(),
            P_k1: controller.P_k1(),
            I_k1: controller.I_k1(),
            D_k1: controller.D_k1(),
            r_k1: controller.r_k1(),
            y_k1: controller.y_k1(),
            y_k2: controller.y_k2(),
            t_k1: controller.t_k1(),
        }
    }
}

/// The saturation function restricts the input `v` to the interval
/// `[u_low, u_high]`.
fn sat(v: f64, u_low: f64, u_high: f64) -> f64 {
    if v < u_low {
        u_low
    } else if v > u_high {
        u_high
    } else {
        v
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ControllerError {
    #[error("database error {0}")]
    Database(#[from] ControllerDbError),

    #[error("error extracting figment config {0}")]
    Figment(#[from] figment::Error),

    #[error("found no controller")]
    NotFound,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Key {
    event_source_id: Uuid,
    plugin_id: Uuid,
}

impl Key {
    fn new(event_source_id: Uuid, plugin_id: Uuid) -> Self {
        Self { event_source_id, plugin_id }
    }

    fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }

    fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }
}

pub struct Controller {
    cache: Cache<Key, PidController>,
    db: ControllerDb,
}

impl Controller {
    pub async fn new(
        cache_capacity: u64,
        cache_ttl: Duration,
    ) -> Result<Self, ControllerError> {
        let listener = move |k, v, cause| {
            todo!()
        };

        let cache = Cache::builder()
            .max_capacity(cache_capacity)
            .time_to_live(cache_ttl)
            .eviction_listener_with_queued_delivery_mode(listener)
            .build();

        let db = ControllerDb::init_with_config(
            Figment::new()
                .merge(Env::prefixed("CONTROLLER_DB_"))
                .extract()?
        ).await?;

        Ok(Self { cache, db })
    }

    pub async fn throttling_quota_for_event_source(
        &self,
        event_source_id: Uuid,
        plugin_id: Uuid,
    ) -> Result<f64, ControllerError> {
        match self.cache.get(&Key::new(event_source_id, plugin_id)) {
            Some(controller) => Ok(controller.control_output()),
            None => {
                // cache miss so we'll retrieve the controller from db
                if let Some(controller) = self.db.get_controller(
                    event_source_id,
                    plugin_id
                ).await? {
                    let pid_controller = PidController::from(controller);
                    let retval = pid_controller.control_output();

                    self.cache.insert(
                        Key::new(event_source_id, plugin_id),
                        pid_controller
                    ).await;

                    Ok(retval)
                } else {
                    Err(ControllerError::NotFound)
                }
            }
        }
    }
}
