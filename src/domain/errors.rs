use crate::error_chain;

error_chain! {
    #[derive(thiserror::Error)]
    pub enum JoinGroupError {
        #[error("{0}")]
        NotFound(&'static str),
        #[error("Invalid token.")]
        Unauthorized(&'static str),
        #[error("User is already a member.")]
        Conflict(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum LoginError {
        #[error("{0}")]
        Validation(&'static str),
        #[error("Credentials are invalid.")]
        InvalidCredentials(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum LogoutError {
        #[error("User is not recognized.")]
        Unauthenticated(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GetUsersError {
        #[error("User is not recognized.")]
        Unauthenticated(),
        #[error("User is not an administrator.")]
        Unauthorized(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GetAllGroupsError {
        #[error("User is not recognized.")]
        Unauthenticated(),
        #[error("User is not an administrator.")]
        Unauthorized(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum CredentialServiceError {
        #[error("Credentials are invalid.")]
        InvalidCredentials(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum ChangeMemberColorError {
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GenerateGroupTokenError {
        #[error("{0}")]
        NotFound(&'static str),
        #[error("User is not admin.")]
        Unauthorized(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum CreateGroupError {
        #[error("{0}")]
        Validation(&'static str),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum CreateUserError {
        #[error("Email already in use.")]
        Conflict(),
        #[error("{0}")]
        Validation(&'static str),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum DeleteGroupError {
        #[error("group not found.")]
        NotFound(),
        #[error("User is not group admin.")]
        Unauthorized(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum CreateExpenseError {
        #[error("{0}")]
        Validation(&'static str),
        #[error("Group not found.")]
        GroupNotFound(),
        #[error("You are not authorized to create an expense in this group.")]
        Unauthorized(),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum SettlementError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum UpdateExpenseError {
        #[error("{0}")]
        Validation(&'static str),
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum DeleteExpenseError {
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum EventHandlerError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum NotifyError {
        #[error("{0}")]
        NotFound(&'static str),
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GetGroupError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GetGroupsError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GetSettlementsError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GetExpensesError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
        #[error("{0}")]
        NotFound(&'static str),
        #[error("{0}")]
        Unauthorized(&'static str),
        #[error("User is not recognized.")]
        Unauthenticated(),
    }
}
