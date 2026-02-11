<?php

function pam_auth(
    string $username,
    string $password,
    ?string &$error = null,
    bool $check_account_management = true,
    string $service_name = 'php',
) {}

function pam_chpass(
    string $username,
    string $old_password,
    string $new_password,
    string &$error = null,
    string $service_name = 'php',
) {}
