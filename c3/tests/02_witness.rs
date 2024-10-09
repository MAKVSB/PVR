//! Run this file with `cargo test --test 02_witness`.

//! Imagine that you're implementing a website that has various endpoints.
//! When a user visits an endpoint, you have to check their permissions.
//! Below, there is an endpoint `endpoint_admin_dashboard` that should only be accessible to admins.
//! It checks if the user is admin, and if they are not, the access is denied.
//!
//! However, there can be hundreds of such endpoints.
//! What if the programmer forgets to use `is_admin` in one of them and calls
//! `show_admin_dashboard` by accident even for non-admin users?
//!
//! Try to modify the code in a way that it will be **impossible** to forget checking if the user
//! is an admin before calling `show_admin_dashboard`.
//! Try to encode invariants using the type system to achieve that.

use admin::Admin;


struct User {
    id: u32,
}

mod admin {
    use super::User;
    pub struct Admin(User);

    enum MaybeAdmin{
        Admin(Admin),
        Denied,
    }

    pub fn as_admin(user: User) -> MaybeAdmin {
        if is_admin(&user) {
            return MaybeAdmin::Admin(Admin(user))
        }
        MaybeAdmin::Denied 
    }

    pub fn is_admin(user: &User) -> bool {
        return user.id == 0
    }

}
// How to make sure that this function can only be called for admin users?
fn show_admin_dashboard(_: Admin) -> u32 {
    // Do not modify the body of this function below
    // Assume that this function e.g. does not access the DB anymore, and it thus can't check if the
    // user is admin.
    200
}

fn endpoint_admin_dashboard(user: User) -> u32 {
    // What if the user forgets this check?
    // Can we make the code more robust, so that they cannot forget?
    // Can ownership + encapsulation help us somehow?
    show_admin_dashboard(admin::as_admin(user))
}
