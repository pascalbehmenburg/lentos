use actix_http::body::EitherBody;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use askama::Template;
use color_eyre::eyre;

use crate::util::error::Error;

pub struct TemplateResponder<T>(pub T);

impl<T> Responder for TemplateResponder<T>
where
    T: Template,
{
    type Body = EitherBody<String>;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.0.render() {
            Ok(body) => match HttpResponse::Ok()
                .content_type(T::MIME_TYPE)
                .message_body(body)
            {
                Ok(res) => res.map_into_left_body(),
                // building http response failed
                Err(err) => Error::Internal(eyre::eyre!(
                    "Failed building HttpResponse in TemplateResponder: {}",
                    err
                ))
                .error_response()
                .map_into_right_body(),
            },
            // rendering failed
            Err(err) => Error::Internal(err.into())
                .error_response()
                .map_into_right_body(),
        }
        // the AskamaResponse contained an error that is converted to
        // ResponseError here
    }
}
