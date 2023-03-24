use pretty_assertions::assert_eq;
use sudo_test::{Command, Env, User};

use crate::{
    Result, GROUPNAME, SUDOERS_ALL_ALL_NOPASSWD, SUDOERS_ROOT_ALL_NOPASSWD,
    SUDOERS_USER_ALL_NOPASSWD, USERNAME,
};

#[test]
fn root_can_become_another_user_by_name() -> Result<()> {
    let env = Env(SUDOERS_ROOT_ALL_NOPASSWD).user(USERNAME).build()?;

    let expected = Command::new("id").as_user(USERNAME).exec(&env)?.stdout()?;
    let actual = Command::new("sudo")
        .args(["-u", USERNAME, "id"])
        .exec(&env)?
        .stdout()?;

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn root_can_become_another_user_by_uid() -> Result<()> {
    let env = Env(SUDOERS_ROOT_ALL_NOPASSWD).user(USERNAME).build()?;

    let uid = Command::new("id")
        .arg("-u")
        .as_user(USERNAME)
        .exec(&env)?
        .stdout()?
        .parse::<u32>()?;
    let expected = Command::new("id").as_user(USERNAME).exec(&env)?.stdout()?;
    let actual = Command::new("sudo")
        .arg("-u")
        .arg(format!("#{uid}"))
        .arg("id")
        .exec(&env)?
        .stdout()?;

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn user_can_become_another_user() -> Result<()> {
    let invoking_user = USERNAME;
    let another_user = "another_user";
    let env = Env(SUDOERS_USER_ALL_NOPASSWD)
        .user(invoking_user)
        .user(another_user)
        .build()?;

    let expected = Command::new("id")
        .as_user(another_user)
        .exec(&env)?
        .stdout()?;
    let actual = Command::new("sudo")
        .args(["-u", another_user, "id"])
        .as_user(USERNAME)
        .exec(&env)?
        .stdout()?;

    assert_eq!(expected, actual);

    Ok(())
}

// regression test for memorysafety/sudo-rs#81
#[test]
fn invoking_user_groups_are_lost_when_becoming_another_user() -> Result<()> {
    let invoking_user = USERNAME;
    let another_user = "another_user";
    let env = Env(SUDOERS_USER_ALL_NOPASSWD)
        .group(GROUPNAME)
        .user(User(invoking_user).group(GROUPNAME))
        .user(another_user)
        .build()?;

    let expected = Command::new("id")
        .as_user(another_user)
        .exec(&env)?
        .stdout()?;
    let actual = Command::new("sudo")
        .args(["-u", another_user, "id"])
        .as_user(invoking_user)
        .exec(&env)?
        .stdout()?;

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
#[ignore]
fn can_use_unassigned_user_id() -> Result<()> {
    let expected_uid = 1234;
    let env = Env(SUDOERS_ALL_ALL_NOPASSWD).user(USERNAME).build()?;

    for user in ["root", USERNAME] {
        let actual = Command::new("sudo")
            .arg("-u")
            .arg(format!("#{expected_uid}"))
            .args(["id", "-u"])
            .as_user(user)
            .exec(&env)?
            .stdout()?
            .parse::<u32>()?;

        assert_eq!(expected_uid, actual);
    }

    Ok(())
}

#[test]
fn user_does_not_exist() -> Result<()> {
    let env = Env(SUDOERS_ROOT_ALL_NOPASSWD).build()?;

    let output = Command::new("sudo")
        .args(["-u", "ghost", "true"])
        .exec(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    if sudo_test::is_original_sudo() {
        assert_contains!(output.stderr(), "unknown user: ghost");
    }

    Ok(())
}