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
    UserNotFound(u32),
    BookNotFound(u32),
    DataDuplicated(u32),
    OverBorrowingLimit,
    BookNotReturned,
    BookNotBorrowed,
    SystemError(String),
    ExceedLimit(u32),
    NotPossibleToDelete,
    ExceedLimitInParallel(u32),
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
            BibErrorResponse::UserNotFound(id) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 106,
                    message: format!("ID({})が見つかりません", id),
                    reason: String::new(),
                })
            }
            BibErrorResponse::BookNotFound(id) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 107,
                    message: format!("該当図書が見つかりません(ID = {})", id),
                    reason: String::new(),
                })
            }
            BibErrorResponse::DataDuplicated(id) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 108,
                    message: format!("該当するデータが複数存在しています({})", id),
                    reason: String::new(),
                })
            }
            BibErrorResponse::OverBorrowingLimit => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 109,
                    message: String::from("貸出できる上限を超えています"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::BookNotReturned => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 110,
                    message: String::from("この本は返却されていません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::BookNotBorrowed => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 111,
                    message: String::from("この本は貸出されていません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::SystemError(reason) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 112,
                    message: String::from("システムエラーが発生しました"),
                    reason: reason.to_string(),
                })
            }
            BibErrorResponse::ExceedLimit(id) => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 113,
                    message: format!("追加できる上限を超えています({})", id),
                    reason: String::new(),
                })
            }
            BibErrorResponse::NotPossibleToDelete => {
                HttpResponse::build(self.status_code()).json(BibResponseBody {
                    success: false,
                    errcode: 114,
                    message: String::from("未返却処理があるため、削除できません"),
                    reason: String::new(),
                })
            }
            BibErrorResponse::ExceedLimitInParallel(id) => HttpResponse::build(self.status_code())
                .json(BibResponseBody {
                    success: false,
                    errcode: 115,
                    message: format!("一度に追加できる上限を超えています({})", id),
                    reason: String::new(),
                }),
        }
    }
}
