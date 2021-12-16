use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;
use serde_json::to_string_pretty;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize)]
pub struct BibResponseBody {
    pub success: bool,
    pub errcode: u16,
    pub message: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub enum BibErrorResponse {
    NotImplemented,
    NotAuthorized,
    LoginFailed,
    DbConnectionError(String),
    InvalidArgument(String),
    DataNotFound(String),
    DataDuplicated,
    OverBorrowingLimit,
    BookNotReturned,
    BookNotBorrowed,
    SystemError(String),
}

impl Display for BibErrorResponse {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl actix_web::error::ResponseError for BibErrorResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
    fn error_response(&self) -> HttpResponse {
        match &*self {
            BibErrorResponse::NotImplemented => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 100,
                    message: String::from("この機能は対応していません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::NotAuthorized => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 101,
                    message: String::from("このアクセスは認証されていません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::LoginFailed => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 102,
                    message: String::from("ログインに失敗しました"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::DbConnectionError(reason) => HttpResponse::build(self.status_code())
                .json(BibResponseBody {
                    success: false,
                    errcode: 103,
                    message: String::from("データベースに接続できません"),
                    reason: reason.to_string(),
                }),
            BibErrorResponse::InvalidArgument(reason) => HttpResponse::build(self.status_code())
                .json(BibResponseBody {
                    success: false,
                    errcode: 104,
                    message: String::from("指定されてパラメータが正しくありません"),
                    reason: reason.to_string(),
                }),
            BibErrorResponse::DataNotFound(reason) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 105,
                    message: String::from("データが見つかりません"),
                    reason: reason.to_string(),
                })
            }
            BibErrorResponse::DataDuplicated => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 106,
                    message: String::from("該当するデータが複数存在しています"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::OverBorrowingLimit => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 107,
                    message: String::from("貸出できる上限を超えています"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::BookNotReturned => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 108,
                    message: String::from("この本は返却されていません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::BookNotBorrowed => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 109,
                    message: String::from("この本は貸出されていません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::SystemError(reason) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 110,
                    message: String::from("システムエラーが発生しました"),
                    reason: reason.to_string(),
                })
            }
        }
    }
}
