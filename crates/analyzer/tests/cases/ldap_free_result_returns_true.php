<?php

declare(strict_types=1);

function free_ldap_result(LDAP\Result $result): true
{
    return ldap_free_result($result);
}
