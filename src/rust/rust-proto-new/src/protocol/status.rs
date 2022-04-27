use tonic::{
    Code as GrpcCode,
    Status as GrpcStatus,
};

/// Status codes used by [`Status`].
///
/// These variants closely mirror the [gRPC status codes].
///
/// [gRPC status codes]: https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Code {
    /// The operation completed successfully.
    Ok = 0,

    /// Unknown error.
    Unknown = 1,

    /// Client specified an invalid argument.
    InvalidArgument = 2,

    /// Some requested entity was not found.
    NotFound = 3,

    /// Some entity that we attempted to create already exists.
    AlreadyExists = 4,

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 5,

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 6,

    /// Operation is not implemented or not supported.
    Unimplemented = 7,

    /// Internal error.
    Internal = 8,

    /// The service is currently unavailable.
    Unavailable = 9,

    /// The request does not have valid authentication credentials
    Unauthenticated = 10,
}

impl Code {
    /// Get description of this `Code`.
    /// ```
    /// fn make_rpc_request() -> rust_proto_new::protocol::status::Code {
    ///     // ...
    ///     rust_proto_new::protocol::status::Code::Ok
    /// }
    /// let code = make_rpc_request();
    /// println!("Operation completed. Human readable description: {}", code.description());
    /// ```
    /// If you only need description in `println`, `format`, `log` and other
    /// formatting contexts, you may want to use `Display` impl for `Code`
    /// instead.
    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Unknown => "Unknown error",
            Code::InvalidArgument => "Client specified an invalid argument",
            Code::NotFound => "Some requested entity was not found",
            Code::AlreadyExists => "Some entity that we attempted to create already exists",
            Code::PermissionDenied => {
                "The caller does not have permission to execute the specified operation"
            }
            Code::FailedPrecondition => {
                "The system is not in a state required for the operation's execution"
            }
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
            Code::Unavailable => "The service is currently unavailable",
            Code::Unauthenticated => "The request does not have valid authentication credentials",
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.description(), f)
    }
}

impl From<Code> for GrpcCode {
    fn from(code: Code) -> Self {
        match code {
            Code::Ok => GrpcCode::Ok,
            Code::Unknown => GrpcCode::Unknown,
            Code::InvalidArgument => GrpcCode::InvalidArgument,
            Code::NotFound => GrpcCode::NotFound,
            Code::AlreadyExists => GrpcCode::AlreadyExists,
            Code::PermissionDenied => GrpcCode::PermissionDenied,
            Code::FailedPrecondition => GrpcCode::FailedPrecondition,
            Code::Unimplemented => GrpcCode::Unimplemented,
            Code::Internal => GrpcCode::Internal,
            Code::Unavailable => GrpcCode::Unavailable,
            Code::Unauthenticated => GrpcCode::Unauthenticated,
        }
    }
}

impl From<GrpcCode> for Code {
    fn from(code: GrpcCode) -> Self {
        match code {
            GrpcCode::Ok => Code::Ok,
            GrpcCode::Cancelled => Code::Internal,
            GrpcCode::Unknown => Code::Unknown,
            GrpcCode::InvalidArgument => Code::InvalidArgument,
            GrpcCode::DeadlineExceeded => Code::Internal,
            GrpcCode::NotFound => Code::NotFound,
            GrpcCode::AlreadyExists => Code::AlreadyExists,
            GrpcCode::PermissionDenied => Code::PermissionDenied,
            GrpcCode::ResourceExhausted => Code::Internal,
            GrpcCode::FailedPrecondition => Code::FailedPrecondition,
            GrpcCode::Aborted => Code::Internal,
            GrpcCode::OutOfRange => Code::Internal,
            GrpcCode::Unimplemented => Code::Unimplemented,
            GrpcCode::Internal => Code::Internal,
            GrpcCode::Unavailable => Code::Unavailable,
            GrpcCode::DataLoss => Code::Internal,
            GrpcCode::Unauthenticated => Code::Unauthenticated,
        }
    }
}

pub struct Status {
    /// The gRPC status code, found in the `grpc-status` header.
    code: Code,
    /// A relevant error message, found in the `grpc-message` header.
    message: String,
}

impl Status {
    /// Create a new `Status` with the associated code and message.
    pub fn new(code: Code, message: impl Into<String>) -> Status {
        Status {
            code,
            message: message.into(),
        }
    }

    /// The operation completed successfully.
    pub fn ok(message: impl Into<String>) -> Status {
        Status::new(Code::Ok, message)
    }

    /// Unknown error. An example of where this error may be returned is if a
    /// `Status` value received from another address space belongs to an error-space
    /// that is not known in this address space. Also errors raised by APIs that
    /// do not return enough error information may be converted to this error.
    pub fn unknown(message: impl Into<String>) -> Status {
        Status::new(Code::Unknown, message)
    }

    /// Client specified an invalid argument. Note that this differs from
    /// `FailedPrecondition`. `InvalidArgument` indicates arguments that are
    /// problematic regardless of the state of the system (e.g., a malformed file
    /// name).
    pub fn invalid_argument(message: impl Into<String>) -> Status {
        Status::new(Code::InvalidArgument, message)
    }

    /// Some requested entity (e.g., file or directory) was not found.
    pub fn not_found(message: impl Into<String>) -> Status {
        Status::new(Code::NotFound, message)
    }

    /// Some entity that we attempted to create (e.g., file or directory) already
    /// exists.
    pub fn already_exists(message: impl Into<String>) -> Status {
        Status::new(Code::AlreadyExists, message)
    }

    /// The caller does not have permission to execute the specified operation.
    /// `PermissionDenied` must not be used for rejections caused by exhausting
    /// some resource (use `ResourceExhausted` instead for those errors).
    /// `PermissionDenied` must not be used if the caller cannot be identified
    /// (use `Unauthenticated` instead for those errors).
    pub fn permission_denied(message: impl Into<String>) -> Status {
        Status::new(Code::PermissionDenied, message)
    }

    /// Operation was rejected because the system is not in a state required for
    /// the operation's execution. For example, directory to be deleted may be
    /// non-empty, an rmdir operation is applied to a non-directory, etc.
    ///
    /// A litmus test that may help a service implementor in deciding between
    /// `FailedPrecondition`, `Aborted`, and `Unavailable`:
    /// (a) Use `Unavailable` if the client can retry just the failing call.
    /// (b) Use `Aborted` if the client should retry at a higher-level (e.g.,
    ///     restarting a read-modify-write sequence).
    /// (c) Use `FailedPrecondition` if the client should not retry until the
    ///     system state has been explicitly fixed.  E.g., if an "rmdir" fails
    ///     because the directory is non-empty, `FailedPrecondition` should be
    ///     returned since the client should not retry unless they have first
    ///     fixed up the directory by deleting files from it.
    pub fn failed_precondition(message: impl Into<String>) -> Status {
        Status::new(Code::FailedPrecondition, message)
    }

    /// Operation is not implemented or not supported/enabled in this service.
    pub fn unimplemented(message: impl Into<String>) -> Status {
        Status::new(Code::Unimplemented, message)
    }

    /// Internal errors. Means some invariants expected by underlying system has
    /// been broken. If you see one of these errors, something is very broken.
    pub fn internal(message: impl Into<String>) -> Status {
        Status::new(Code::Internal, message)
    }

    /// The service is currently unavailable.  This is a most likely a transient
    /// condition and may be corrected by retrying with a back-off.
    ///
    /// See litmus test above for deciding between `FailedPrecondition`,
    /// `Aborted`, and `Unavailable`.
    pub fn unavailable(message: impl Into<String>) -> Status {
        Status::new(Code::Unavailable, message)
    }

    /// The request does not have valid authentication credentials for the
    /// operation.
    pub fn unauthenticated(message: impl Into<String>) -> Status {
        Status::new(Code::Unauthenticated, message)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "status: {:?}, message: {:?}", self.code, self.message,)
    }
}

impl std::fmt::Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A manual impl to reduce the noise of frequently empty fields.
        let mut builder = f.debug_struct("Status");

        builder.field("code", &self.code);

        if !self.message.is_empty() {
            builder.field("message", &self.message);
        }

        builder.finish()
    }
}

impl From<Status> for GrpcStatus {
    fn from(status: Status) -> Self {
        GrpcStatus::new(status.code.into(), status.message)
    }
}

impl From<GrpcStatus> for Status {
    fn from(status: GrpcStatus) -> Self {
        Status::new(status.code().into(), status.message())
    }
}
