use std::{
    convert::Infallible,
    ops::{FromResidual, Try},
};

use actix_http::body::EitherBody;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};

use super::error::Error;

#[derive(
    Clone,
    Debug,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
)]
pub struct ErrorOr<T, E = Error>(pub Result<T, E>);

impl<T, E> FromResidual<Result<Infallible, E>> for ErrorOr<T>
where
    E: Into<Error>,
{
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(error) => Self(Err(error.into())),
        }
    }
}

impl<T> Try for ErrorOr<T> {
    type Output = T;
    type Residual = Result<Infallible, Error>;

    fn from_output(output: Self::Output) -> Self {
        Self(Ok(output))
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(output) => std::ops::ControlFlow::Continue(output),
            Err(error) => std::ops::ControlFlow::Break(Err(error)),
        }
    }
}

// // enables one to use this syntax
// // some_error_func()
// //.map_err(|| Error::External(StatusCode::BadRequest, "Meme"))
// //.into()
// impl<T> From<Result<T, Error>> for ErrorOr<T> {
//     fn from(result: Result<T, Error>) -> Self {
//         match result {
//             Ok(output) => Self(Ok(output)),
//             Err(error) => Self(Err(error)),
//         }
//     }
// }

// //allows the following refactor:
// //
// //before:
// // some_error_func() // is Result<T, eyre::Error>
// //.map_err(Into::into) // is eyre::Error
// //.map_err(Error::Internal)
// //.into()
// //
// //after:
// // some_error_func()
// //.into()
// impl<T, E> From<Result<T, E>> for ErrorOr<T>
// where
//     E: Into<eyre::Error>,
// {
//     fn from(result: Result<T, E>) -> Self {
//         match result {
//             Ok(output) => Self(Ok(output)),
//             Err(error) => {
//                 let error = error.into();
//                 Self(Err(Error::Internal(error)))
//             }
//         }
//     }
// }

// allows the following:
// some_error_func() -> ErrorOr<T> {
//   let foo: T = bar();
//   foo.into()
// }
impl<T> From<T> for ErrorOr<T> {
    fn from(output: T) -> Self {
        Self(Ok(output))
    }
}

impl<T: Responder, E: Into<Error>> Responder for ErrorOr<T, E> {
    type Body = EitherBody<T::Body>;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.0 {
            Ok(output) => output.respond_to(req).map_into_left_body(),
            Err(error) => {
                let e: Error = error.into();
                e.error_response().map_into_right_body()
            }
        }
    }
}
