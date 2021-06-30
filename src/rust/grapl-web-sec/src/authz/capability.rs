use crate::authn::AuthenticatedUser;
use actix_web::{FromRequest, HttpRequest};
use std::future::Future;
use actix_web::dev::Payload;
use std::ops::Deref;
use std::pin::Pin;
use crate::error::WebSecError;

pub trait CapabilityT: Sized {
    type Resource: Clone;

    fn get_capability(auth: &AuthenticatedUser, resource: &Self::Resource) -> Result<Self, WebSecError>;
}

pub struct Capability<C: CapabilityT> {
    inner: C
}

impl<C: CapabilityT> Deref for Capability<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C: CapabilityT<Resource = ()>> FromRequest for Capability<C> {
    type Error = WebSecError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let capability_req = req.clone();

        Box::pin(async move {
            let auth_user = AuthenticatedUser::from_request(&capability_req, &mut Payload::None).await?;

            let inner = C::get_capability(&auth_user, &())?;

            Ok(Capability { inner })
        })
    }
}