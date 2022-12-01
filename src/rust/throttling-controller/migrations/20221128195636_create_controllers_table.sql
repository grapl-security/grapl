CREATE TABLE IF NOT EXISTS controllers
(
    event_source_id     uuid                    NOT NULL,
    plugin_id           uuid                    NOT NULL,
    is_generator        bool                    NOT NULL,
    K                   double precision        NOT NULL,
    T_i                 double precision        NOT NULL,
    T_d                 double precision        NOT NULL,
    T_t                 double precision        NOT NULL,
    N                   double precision        NOT NULL,
    b                   double precision        NOT NULL,
    P_k1                double precision        NOT NULL,
    I_k1                double precision        NOT NULL,
    D_k1                double precision        NOT NULL,
    r_k1                double precision        NOT NULL,
    y_k1                double precision        NOT NULL,
    y_k2                double precision        NOT NULL,
    t_k1                timestamptz             NOT NULL,
    PRIMARY KEY (event_source_id, plugin_id)
);
