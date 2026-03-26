<?php

namespace {
    use LDAP\Result;

    /**
     * @param-out array $controls
     */
    function ldap_exop_passwd(
        LDAP\Connection $ldap,
        string $user = '',
        #[SensitiveParameter] string $old_password = '',
        #[SensitiveParameter] string $new_password = '',
        &$controls = null,
    ): string|bool {}

    function ldap_exop_refresh(LDAP\Connection $ldap, string $dn, int $ttl): int|false {}

    function ldap_exop_whoami(LDAP\Connection $ldap): string|false {}

    /**
     * @param-out string $response_data
     * @param-out string $response_oid
     */
    function ldap_exop(
        LDAP\Connection $ldap,
        string $request_oid,
        ?string $request_data,
        ?array $controls = null,
        &$response_data,
        &$response_oid,
    ): LDAP\Result|bool {}

    /**
     * @param-out string $response_data
     * @param-out string $response_oid
     */
    function ldap_parse_exop(
        LDAP\Connection $ldap,
        LDAP\Result $result,
        &$response_data = null,
        &$response_oid = null,
    ): bool {}

    function ldap_8859_to_t61(string $value): string {}

    function ldap_t61_to_8859(string $value): string {}

    function ldap_connect(?string $uri, int $port = 389): LDAP\Connection|false {}

    function ldap_close(LDAP\Connection $ldap): bool {}

    function ldap_bind(LDAP\Connection $ldap, ?string $dn, #[SensitiveParameter] ?string $password): bool {}

    function ldap_bind_ext(
        LDAP\Connection $ldap,
        ?string $dn,
        ?string $password,
        ?array $controls = null,
    ): LDAP\Result|false {}

    /**
     * @param null|string $dn
     * @param null|string $password
     * @param null|string $mech
     * @param null|string $realm
     * @param null|string $authc_id
     * @param null|string $authz_id
     * @param null|string $props
     */
    function ldap_sasl_bind(
        LDAP\Connection $ldap,
        ?string $dn = null,
        #[SensitiveParameter] ?string $password = null,
        ?string $mech = null,
        ?string $realm = null,
        ?string $authc_id = null,
        ?string $authz_id = null,
        ?string $props = null,
    ): bool {}

    function ldap_unbind(LDAP\Connection $ldap): bool {}

    function ldap_read(
        LDAP\Connection $ldap,
        array|string $base,
        array|string $filter,
        array $attributes = [],
        int $attributes_only = 0,
        int $sizelimit = -1,
        int $timelimit = -1,
        int $deref = 0,
        ?array $controls = null,
    ): LDAP\Result|array|false {}

    function ldap_list(
        LDAP\Connection $ldap,
        array|string $base,
        array|string $filter,
        array $attributes = [],
        int $attributes_only = 0,
        int $sizelimit = -1,
        int $timelimit = -1,
        int $deref = 0,
        ?array $controls = null,
    ): LDAP\Result|array|false {}

    function ldap_search(
        LDAP\Connection $ldap,
        array|string $base,
        array|string $filter,
        array $attributes = [],
        int $attributes_only = 0,
        int $sizelimit = -1,
        int $timelimit = -1,
        int $deref = 0,
        ?array $controls = null,
    ): LDAP\Result|array|false {}

    function ldap_free_result(Result $result): bool {}

    function ldap_count_entries(LDAP\Connection $ldap, LDAP\Result $result): int {}

    function ldap_first_entry(LDAP\Connection $ldap, LDAP\Result $result): LDAP\ResultEntry|false {}

    function ldap_next_entry(LDAP\Connection $ldap, LDAP\ResultEntry $entry): LDAP\ResultEntry|false {}

    function ldap_get_entries(LDAP\Connection $ldap, LDAP\Result $result): array|false {}

    function ldap_first_attribute(LDAP\Connection $ldap, LDAP\ResultEntry $entry): string|false {}

    function ldap_next_attribute(LDAP\Connection $ldap, LDAP\ResultEntry $entry): string|false {}

    function ldap_get_attributes(LDAP\Connection $ldap, LDAP\ResultEntry $entry): array {}

    function ldap_get_values(LDAP\Connection $ldap, LDAP\ResultEntry $entry, string $attribute): array|false {}

    function ldap_get_values_len(LDAP\Connection $ldap, LDAP\ResultEntry $entry, string $attribute): array|false {}

    /**
     * @return string|false
     */
    function ldap_get_dn(LDAP\Connection $ldap, LDAP\ResultEntry $entry): string|false {}

    /**
     * @return array{count: int}|false
     */
    function ldap_explode_dn(string $dn, int $with_attrib): array|false {}

    function ldap_dn2ufn(string $dn): string|false {}

    function ldap_add(LDAP\Connection $ldap, string $dn, array $entry, ?array $controls = null): bool {}

    function ldap_add_ext(
        LDAP\Connection $ldap,
        string $dn,
        array $entry,
        ?array $controls = null,
    ): LDAP\Result|false {}

    function ldap_delete(LDAP\Connection $ldap, string $dn, ?array $controls = null): bool {}

    function ldap_delete_ext(LDAP\Connection $ldap, string $dn, ?array $controls = null): LDAP\Result|false {}

    function ldap_modify(LDAP\Connection $ldap, string $dn, array $entry, ?array $controls = null): bool {}

    function ldap_mod_add(LDAP\Connection $ldap, string $dn, array $entry, ?array $controls = null): bool {}

    function ldap_mod_add_ext(
        LDAP\Connection $ldap,
        string $dn,
        array $entry,
        ?array $controls = null,
    ): LDAP\Result|false {}

    function ldap_mod_replace(LDAP\Connection $ldap, string $dn, array $entry, ?array $controls = null): bool {}

    function ldap_mod_replace_ext(
        LDAP\Connection $ldap,
        string $dn,
        array $entry,
        ?array $controls = null,
    ): LDAP\Result|false {}

    function ldap_mod_del(LDAP\Connection $ldap, string $dn, array $entry, ?array $controls = null): bool {}

    function ldap_mod_del_ext(
        LDAP\Connection $ldap,
        string $dn,
        array $entry,
        ?array $controls = null,
    ): LDAP\Result|false {}

    function ldap_errno(LDAP\Connection $ldap): int {}

    function ldap_err2str(int $errno): string {}

    function ldap_error(LDAP\Connection $ldap): string {}

    function ldap_compare(
        LDAP\Connection $ldap,
        string $dn,
        string $attribute,
        string $value,
        ?array $controls = null,
    ): int|bool {}

    function ldap_rename(
        LDAP\Connection $ldap,
        string $dn,
        string $new_rdn,
        string $new_parent,
        bool $delete_old_rdn,
        ?array $controls = null,
    ): bool {}

    function ldap_rename_ext(
        LDAP\Connection $ldap,
        string $dn,
        string $new_rdn,
        string $new_parent,
        bool $delete_old_rdn,
        ?array $controls = null,
    ): LDAP\Result|false {}

    function ldap_get_option(LDAP\Connection $ldap, int $option, &$value = null): bool {}

    function ldap_set_option(?LDAP\Connection $ldap, int $option, $value): bool {}

    function ldap_first_reference(LDAP\Connection $ldap, LDAP\Result $result): LDAP\ResultEntry|false {}

    function ldap_next_reference(LDAP\Connection $ldap, LDAP\ResultEntry $entry): LDAP\ResultEntry|false {}

    /**
     * @param-out array $referrals
     */
    function ldap_parse_reference(LDAP\Connection $ldap, LDAP\ResultEntry $entry, &$referrals): bool {}

    /**
     * @param-out int &$error_code
     * @param-out string &$matched_dn
     * @param-out string &$error_message
     * @param-out array &$referrals
     * @param-out array &$controls
     */
    function ldap_parse_result(
        LDAP\Connection $ldap,
        LDAP\Result $result,
        &$error_code,
        &$matched_dn,
        &$error_message,
        &$referrals,
        &$controls = null,
    ): bool {}

    function ldap_start_tls(LDAP\Connection $ldap): bool {}

    function ldap_set_rebind_proc(LDAP\Connection $ldap, ?callable $callback): bool {}

    function ldap_escape(string $value, string $ignore = '', int $flags = 0): string {}

    function ldap_modify_batch(
        LDAP\Connection $ldap,
        string $dn,
        array $modifications_info,
        ?array $controls = null,
    ): bool {}

    function ldap_count_references(LDAP\Connection $ldap, LDAP\Result $result): int {}

    function ldap_exop_sync(
        LDAP\Connection $ldap,
        string $request_oid,
        ?string $request_data = null,
        ?array $controls = null,
        &$response_data = null,
        &$response_oid = null,
    ): Result|bool {}

    const LDAP_ESCAPE_FILTER = 1;

    const LDAP_ESCAPE_DN = 2;

    const LDAP_DEREF_NEVER = 0;

    const LDAP_DEREF_SEARCHING = 1;

    const LDAP_DEREF_FINDING = 2;

    const LDAP_DEREF_ALWAYS = 3;

    const LDAP_MODIFY_BATCH_REMOVE = 2;

    const LDAP_MODIFY_BATCH_ADD = 1;

    const LDAP_MODIFY_BATCH_REMOVE_ALL = 18;

    const LDAP_MODIFY_BATCH_REPLACE = 3;

    const LDAP_OPT_X_TLS_REQUIRE_CERT = 24582;

    const LDAP_OPT_X_TLS_NEVER = 0;

    const LDAP_OPT_X_TLS_HARD = 1;

    const LDAP_OPT_X_TLS_DEMAND = 2;

    const LDAP_OPT_X_TLS_ALLOW = 3;

    const LDAP_OPT_X_TLS_TRY = 4;

    const LDAP_OPT_X_TLS_CERTFILE = 24580;

    const LDAP_OPT_X_TLS_CIPHER_SUITE = 24584;

    const LDAP_OPT_X_TLS_KEYFILE = 24581;

    const LDAP_OPT_X_TLS_DHFILE = 24590;

    const LDAP_OPT_X_TLS_CRLFILE = 24592;

    const LDAP_OPT_X_TLS_RANDOM_FILE = 24585;

    const LDAP_OPT_X_TLS_CRLCHECK = 24587;

    const LDAP_OPT_X_TLS_CRL_NONE = 0;

    const LDAP_OPT_X_TLS_CRL_PEER = 1;

    const LDAP_OPT_X_TLS_CRL_ALL = 2;

    const LDAP_OPT_X_TLS_PROTOCOL_MIN = 24583;

    const LDAP_OPT_X_TLS_PROTOCOL_SSL2 = 512;

    const LDAP_OPT_X_TLS_PROTOCOL_SSL3 = 768;

    const LDAP_OPT_X_TLS_PROTOCOL_TLS1_0 = 769;

    const LDAP_OPT_X_TLS_PROTOCOL_TLS1_1 = 770;

    const LDAP_OPT_X_TLS_PROTOCOL_TLS1_2 = 771;

    const LDAP_OPT_X_TLS_PACKAGE = 24593;

    const LDAP_OPT_X_KEEPALIVE_IDLE = 25344;

    const LDAP_OPT_X_KEEPALIVE_PROBES = 25345;

    const LDAP_OPT_X_KEEPALIVE_INTERVAL = 25346;

    const LDAP_OPT_X_SASL_USERNAME = 24844;

    const LDAP_OPT_X_SASL_NOCANON = 24843;

    const LDAP_OPT_DEREF = 2;

    const LDAP_OPT_SIZELIMIT = 3;

    const LDAP_OPT_TIMELIMIT = 4;

    const LDAP_OPT_NETWORK_TIMEOUT = 20485;

    const LDAP_OPT_PROTOCOL_VERSION = 17;

    const LDAP_OPT_ERROR_NUMBER = 49;

    const LDAP_OPT_REFERRALS = 8;

    const LDAP_OPT_RESTART = 9;

    const LDAP_OPT_HOST_NAME = 48;

    const LDAP_OPT_ERROR_STRING = 50;

    const LDAP_OPT_MATCHED_DN = 51;

    const LDAP_OPT_SERVER_CONTROLS = 18;

    const LDAP_OPT_CLIENT_CONTROLS = 19;

    const LDAP_OPT_DEBUG_LEVEL = 20481;

    const LDAP_OPT_X_SASL_MECH = 24832;

    const LDAP_OPT_X_SASL_REALM = 24833;

    const LDAP_OPT_X_SASL_AUTHCID = 24834;

    const LDAP_OPT_X_SASL_AUTHZID = 24835;

    const LDAP_OPT_X_TLS_CACERTDIR = 24579;

    const LDAP_OPT_X_TLS_CACERTFILE = 24578;

    const LDAP_MODIFY_BATCH_ATTRIB = 'attrib';

    const LDAP_MODIFY_BATCH_MODTYPE = 'modtype';

    const LDAP_MODIFY_BATCH_VALUES = 'values';

    const LDAP_OPT_TIMEOUT = 20482;

    const LDAP_OPT_DIAGNOSTIC_MESSAGE = 50;

    const LDAP_CONTROL_MANAGEDSAIT = '2.16.840.1.113730.3.4.2';

    const LDAP_CONTROL_PROXY_AUTHZ = '2.16.840.1.113730.3.4.18';

    const LDAP_CONTROL_SUBENTRIES = '1.3.6.1.4.1.4203.1.10.1';

    const LDAP_CONTROL_VALUESRETURNFILTER = '1.2.826.0.1.3344810.2.3';

    const LDAP_CONTROL_ASSERT = '1.3.6.1.1.12';

    const LDAP_CONTROL_PRE_READ = '1.3.6.1.1.13.1';

    const LDAP_CONTROL_POST_READ = '1.3.6.1.1.13.2';

    const LDAP_CONTROL_SORTREQUEST = '1.2.840.113556.1.4.473';

    const LDAP_CONTROL_SORTRESPONSE = '1.2.840.113556.1.4.474';

    const LDAP_CONTROL_PAGEDRESULTS = '1.2.840.113556.1.4.319';

    const LDAP_CONTROL_SYNC = '1.3.6.1.4.1.4203.1.9.1.1';

    const LDAP_CONTROL_SYNC_STATE = '1.3.6.1.4.1.4203.1.9.1.2';

    const LDAP_CONTROL_SYNC_DONE = '1.3.6.1.4.1.4203.1.9.1.3';

    const LDAP_CONTROL_DONTUSECOPY = '1.3.6.1.1.22';

    const LDAP_CONTROL_PASSWORDPOLICYREQUEST = '1.3.6.1.4.1.42.2.27.8.5.1';

    const LDAP_CONTROL_PASSWORDPOLICYRESPONSE = '1.3.6.1.4.1.42.2.27.8.5.1';

    const LDAP_CONTROL_X_INCREMENTAL_VALUES = '1.2.840.113556.1.4.802';

    const LDAP_CONTROL_X_DOMAIN_SCOPE = '1.2.840.113556.1.4.1339';

    const LDAP_CONTROL_X_PERMISSIVE_MODIFY = '1.2.840.113556.1.4.1413';

    const LDAP_CONTROL_X_SEARCH_OPTIONS = '1.2.840.113556.1.4.1340';

    const LDAP_CONTROL_X_TREE_DELETE = '1.2.840.113556.1.4.805';

    const LDAP_CONTROL_X_EXTENDED_DN = '1.2.840.113556.1.4.529';

    const LDAP_CONTROL_VLVREQUEST = '2.16.840.1.113730.3.4.9';

    const LDAP_CONTROL_VLVRESPONSE = '2.16.840.1.113730.3.4.10';

    const LDAP_EXOP_MODIFY_PASSWD = '1.3.6.1.4.1.4203.1.11.1';

    const LDAP_EXOP_REFRESH = '1.3.6.1.4.1.1466.101.119.1';

    const LDAP_EXOP_START_TLS = '1.3.6.1.4.1.1466.20037';

    const LDAP_EXOP_TURN = '1.3.6.1.1.19';

    const LDAP_EXOP_WHO_AM_I = '1.3.6.1.4.1.4203.1.11.3';

    const LDAP_CONTROL_AUTHZID_REQUEST = '2.16.840.1.113730.3.4.16';

    const LDAP_CONTROL_AUTHZID_RESPONSE = '2.16.840.1.113730.3.4.15';

    const LDAP_OPT_X_TLS_PROTOCOL_TLS1_3 = 772;

    const LDAP_OPT_X_TLS_PROTOCOL_MAX = 24603;
}

namespace LDAP {
    final class Connection {}

    final class Result {}

    final class ResultEntry {}
}
