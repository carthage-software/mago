<?php

class Svn
{
    public const NON_RECURSIVE = 1;
    public const DISCOVER_CHANGED_PATHS = 2;
    public const OMIT_MESSAGES = 4;
    public const STOP_ON_COPY = 8;
    public const ALL = 16;
    public const SHOW_UPDATES = 32;
    public const NO_IGNORE = 64;
    public const IGNORE_EXTERNALS = 128;
    public const INITIAL = 1;
    public const HEAD = -1;
    public const BASE = -2;
    public const COMMITTED = -3;
    public const PREV = -4;
    public const UNSPECIFIED = -5;

    public static function checkout() {}

    public static function cat() {}

    public static function ls() {}

    public static function log() {}

    public static function auth_set_parameter() {}

    public static function auth_get_parameter() {}

    public static function client_version() {}

    public static function config_ensure() {}

    public static function diff() {}

    public static function cleanup() {}

    public static function revert() {}

    public static function resolved() {}

    public static function commit() {}

    public static function lock() {}

    public static function unlock() {}

    public static function add() {}

    public static function status() {}

    public static function update() {}

    public static function update2() {}

    public static function import() {}

    public static function info() {}

    public static function export() {}

    public static function copy() {}

    public static function switch() {}

    public static function blame() {}

    public static function delete() {}

    public static function mkdir() {}

    public static function move() {}

    public static function proplist() {}

    public static function propget() {}

    public static function propset() {}

    public static function prop_delete() {}

    public static function revprop_get() {}

    public static function revprop_set() {}

    public static function revprop_delete() {}

    public static function repos_create() {}

    public static function repos_recover() {}

    public static function repos_hotcopy() {}

    public static function repos_open() {}

    public static function repos_fs() {}

    public static function repos_fs_begin_txn_for_commit() {}

    public static function repos_fs_commit_txn() {}
}

class SvnWc
{
    public const NONE = 1;
    public const UNVERSIONED = 2;
    public const NORMAL = 3;
    public const ADDED = 4;
    public const MISSING = 5;
    public const DELETED = 6;
    public const REPLACED = 7;
    public const MODIFIED = 8;
    public const MERGED = 9;
    public const CONFLICTED = 10;
    public const IGNORED = 11;
    public const OBSTRUCTED = 12;
    public const EXTERNAL = 13;
    public const INCOMPLETE = 14;
}

class SvnWcSchedule
{
    public const NORMAL = 0;
    public const ADD = 1;
    public const DELETE = 2;
    public const REPLACE = 3;
}

class SvnNode
{
    public const NONE = 0;
    public const FILE = 1;
    public const DIR = 2;
    public const UNKNOWN = 3;
}

/**
 * @param string $repos
 * @param string $targetpath
 * @param int $revision
 * @param int $flags
 *
 * @return bool
 */
function svn_checkout($repos, $targetpath, $revision = SVN_REVISION_HEAD, $flags = 0) {}

/**
 * @param string $repos_url
 * @param int $revision_no
 *
 * @return string
 */
function svn_cat($repos_url, $revision_no = SVN_REVISION_HEAD) {}

/**
 * @param string $repos_url
 * @param int $revision_no
 * @param bool $recurse
 * @param bool $peg
 *
 * @return array
 */
function svn_ls($repos_url, $revision_no = SVN_REVISION_HEAD, $recurse = false, $peg = false) {}

/**
 * @param string $repos_url
 * @param int $start_revision
 * @param int $end_revision
 * @param int $limit
 * @param int $flags
 *
 * @return array
 */
function svn_log(
    $repos_url,
    $start_revision = null,
    $end_revision = null,
    $limit = 0,
    $flags = SVN_DISCOVER_CHANGED_PATHS | SVN_STOP_ON_COPY,
) {}

/**
 * @param string $key
 * @param string $value
 *
 * @return void
 */
function svn_auth_set_parameter($key, $value) {}

/**
 * @param string $key
 *
 * @return string|null
 */
function svn_auth_get_parameter($key) {}

/**
 * @return string
 */
function svn_client_version() {}

function svn_config_ensure() {}

/**
 * @param string $path1
 * @param int $rev1
 * @param string $path2
 * @param int $rev2
 *
 * @return array
 */
function svn_diff($path1, $rev1, $path2, $rev2) {}

/**
 * @param string $workingdir
 *
 * @return bool
 */
function svn_cleanup($workingdir) {}

/**
 * @param string $path
 * @param bool $recursive
 *
 * @return bool
 */
function svn_revert($path, $recursive = false) {}

function svn_resolved() {}

/**
 * @param string $log
 * @param array $targets
 * @param bool $recursive
 *
 * @return array
 */
function svn_commit($log, array $targets, $recursive = true) {}

function svn_lock() {}

function svn_unlock() {}

/**
 * @param string $path
 * @param bool $recursive
 * @param bool $force
 *
 * @return bool
 */
function svn_add($path, $recursive = true, $force = false) {}

/**
 * @param string $path
 * @param int $flags
 *
 * @return array
 */
function svn_status($path, $flags = 0) {}

/**
 * @param string $path
 * @param int $revno
 * @param bool $recurse
 *
 * @return int|false
 */
function svn_update($path, $revno = SVN_REVISION_HEAD, $recurse = true) {}

/**
 * @param string $path
 * @param string $url
 * @param bool $nonrecursive
 *
 * @return bool
 */
function svn_import($path, $url, $nonrecursive) {}

function svn_info() {}

/**
 * @param string $frompath
 * @param string $topath
 * @param bool $working_copy
 * @param int $revision_no
 *
 * @return bool
 */
function svn_export($frompath, $topath, $working_copy = true, $revision_no = -1) {}

function svn_copy() {}

function svn_switch() {}

/**
 * @param string $repository_url
 * @param int $revision_no
 *
 * @return array
 */
function svn_blame($repository_url, $revision_no = SVN_REVISION_HEAD) {}

/**
 * @param string $path
 * @param bool $force
 *
 * @return bool
 */
function svn_delete($path, $force = false) {}

/**
 * @param string $path
 * @param string $log_message
 *
 * @return bool
 */
function svn_mkdir($path, $log_message = null) {}

/**
 * @param string $src_path
 * @param string $dst_path
 * @param bool $force
 */
function svn_move($src_path, $dst_path, $force = false) {}

/**
 * @param string $path
 * @param bool $recurse
 * @param int $revision
 */
function svn_proplist($path, $recurse = false, $revision) {}

/**
 * @param string $path
 * @param string $property_name
 * @param bool $recurse
 * @param int $revision
 */
function svn_propget($path, $property_name, $recurse = false, $revision) {}

/**
 * @param string $path
 * @param null|array $config
 * @param null|array $fsconfig
 *
 * @return resource
 */
function svn_repos_create($path, ?array $config = null, ?array $fsconfig = null) {}

/**
 * @param string $path
 *
 * @return bool
 */
function svn_repos_recover($path) {}

/**
 * @param string $repospath
 * @param string $destpath
 * @param bool $cleanlogs
 *
 * @return bool
 */
function svn_repos_hotcopy($repospath, $destpath, $cleanlogs) {}

/**
 * @param string $path
 *
 * @return resource
 */
function svn_repos_open($path) {}

/**
 * @param resource $repos
 *
 * @return resource
 */
function svn_repos_fs($repos) {}

/**
 * @param resource $repos
 * @param int $rev
 * @param string $author
 * @param string $log_msg
 *
 * @return resource
 */
function svn_repos_fs_begin_txn_for_commit($repos, $rev, $author, $log_msg) {}

/**
 * @param resource $txn
 *
 * @return int
 */
function svn_repos_fs_commit_txn($txn) {}

/**
 * @param resource $fs
 * @param int $revnum
 *
 * @return resource
 */
function svn_fs_revision_root($fs, $revnum) {}

/**
 * @param resource $fsroot
 * @param string $path
 *
 * @return int
 */
function svn_fs_check_path($fsroot, $path) {}

/**
 * @param resource $fs
 * @param int $revnum
 * @param string $propname
 *
 * @return string
 */
function svn_fs_revision_prop($fs, $revnum, $propname) {}

/**
 * @param resource $fsroot
 * @param string $path
 *
 * @return array
 */
function svn_fs_dir_entries($fsroot, $path) {}

/**
 * @param resource $fsroot
 * @param string $path
 *
 * @return int
 */
function svn_fs_node_created_rev($fsroot, $path) {}

/**
 * @param resource $fs
 *
 * @return int
 */
function svn_fs_youngest_rev($fs) {}

/**
 * @param resource $fsroot
 * @param string $path
 *
 * @return resource
 */
function svn_fs_file_contents($fsroot, $path) {}

/**
 * @param resource $fsroot
 * @param string $path
 *
 * @return int
 */
function svn_fs_file_length($fsroot, $path) {}

/**
 * @param resource $txn
 *
 * @return resource
 */
function svn_fs_txn_root($txn) {}

/**
 * @param resource $root
 * @param string $path
 *
 * @return bool
 */
function svn_fs_make_file($root, $path) {}

/**
 * @param resource $root
 * @param string $path
 *
 * @return bool
 */
function svn_fs_make_dir($root, $path) {}

/**
 * @param resource $root
 * @param string $path
 *
 * @return resource
 */
function svn_fs_apply_text($root, $path) {}

/**
 * @param resource $from_root
 * @param string $from_path
 * @param resource $to_root
 * @param string $to_path
 *
 * @return bool
 */
function svn_fs_copy($from_root, $from_path, $to_root, $to_path) {}

/**
 * @param resource $root
 * @param string $path
 *
 * @return bool
 */
function svn_fs_delete($root, $path) {}

/**
 * @param resource $repos
 * @param int $rev
 *
 * @return resource
 */
function svn_fs_begin_txn2($repos, $rev) {}

/**
 * @param resource $root
 * @param string $path
 *
 * @return bool
 */
function svn_fs_is_dir($root, $path) {}

/**
 * @param resource $root
 * @param string $path
 *
 * @return bool
 */
function svn_fs_is_file($root, $path) {}

/**
 * @param resource $fsroot
 * @param string $path
 * @param string $propname
 *
 * @return string
 */
function svn_fs_node_prop($fsroot, $path, $propname) {}

/**
 * @param resource $root
 * @param string $path
 * @param string $name
 * @param string $value
 *
 * @return bool
 */
function svn_fs_change_node_prop($root, $path, $name, $value) {}

/**
 * @param resource $root1
 * @param string $path1
 * @param resource $root2
 * @param string $path2
 *
 * @return bool
 */
function svn_fs_contents_changed($root1, $path1, $root2, $path2) {}

/**
 * @param resource $root1
 * @param string $path1
 * @param resource $root2
 * @param string $path2
 *
 * @return bool
 */
function svn_fs_props_changed($root1, $path1, $root2, $path2) {}

/**
 * @param resource $txn
 * @return bool
 */
function svn_fs_abort_txn($txn) {}

const SVN_AUTH_PARAM_DEFAULT_USERNAME = 'svn:auth:username';

const SVN_AUTH_PARAM_DEFAULT_PASSWORD = 'svn:auth:password';

const SVN_AUTH_PARAM_NON_INTERACTIVE = 'svn:auth:non-interactive';

const SVN_AUTH_PARAM_DONT_STORE_PASSWORDS = 'svn:auth:dont-store-passwords';

const SVN_AUTH_PARAM_NO_AUTH_CACHE = 'svn:auth:no-auth-cache';

const SVN_AUTH_PARAM_SSL_SERVER_FAILURES = 'svn:auth:ssl:failures';

const SVN_AUTH_PARAM_SSL_SERVER_CERT_INFO = 'svn:auth:ssl:cert-info';

const SVN_AUTH_PARAM_CONFIG = 'svn:auth:config-category-servers';

const SVN_AUTH_PARAM_SERVER_GROUP = 'svn:auth:server-group';

const SVN_AUTH_PARAM_CONFIG_DIR = 'svn:auth:config-dir';

const PHP_SVN_AUTH_PARAM_IGNORE_SSL_VERIFY_ERRORS = 'php:svn:auth:ignore-ssl-verify-errors';

const SVN_FS_CONFIG_FS_TYPE = 'fs-type';

const SVN_FS_TYPE_BDB = 'bdb';

const SVN_FS_TYPE_FSFS = 'fsfs';

const SVN_PROP_REVISION_DATE = 'svn:date';

const SVN_PROP_REVISION_ORIG_DATE = 'svn:original-date';

const SVN_PROP_REVISION_AUTHOR = 'svn:author';

const SVN_PROP_REVISION_LOG = 'svn:log';

const SVN_REVISION_INITIAL = 1;

const SVN_REVISION_HEAD = -1;

const SVN_REVISION_BASE = -2;

const SVN_REVISION_COMMITTED = -3;

const SVN_REVISION_PREV = -4;

const SVN_REVISION_UNSPECIFIED = -5;

const SVN_NON_RECURSIVE = 1;

const SVN_DISCOVER_CHANGED_PATHS = 2;

const SVN_OMIT_MESSAGES = 4;

const SVN_STOP_ON_COPY = 8;

const SVN_ALL = 16;

const SVN_SHOW_UPDATES = 32;

const SVN_NO_IGNORE = 64;

const SVN_WC_STATUS_NONE = 1;

const SVN_WC_STATUS_UNVERSIONED = 2;

const SVN_WC_STATUS_NORMAL = 3;

const SVN_WC_STATUS_ADDED = 4;

const SVN_WC_STATUS_MISSING = 5;

const SVN_WC_STATUS_DELETED = 6;

const SVN_WC_STATUS_REPLACED = 7;

const SVN_WC_STATUS_MODIFIED = 8;

const SVN_WC_STATUS_MERGED = 9;

const SVN_WC_STATUS_CONFLICTED = 10;

const SVN_WC_STATUS_IGNORED = 11;

const SVN_WC_STATUS_OBSTRUCTED = 12;

const SVN_WC_STATUS_EXTERNAL = 13;

const SVN_WC_STATUS_INCOMPLETE = 14;

const SVN_NODE_NONE = 0;

const SVN_NODE_FILE = 1;

const SVN_NODE_DIR = 2;

const SVN_NODE_UNKNOWN = 3;

const SVN_WC_SCHEDULE_NORMAL = 0;

const SVN_WC_SCHEDULE_ADD = 1;

const SVN_WC_SCHEDULE_DELETE = 2;

const SVN_WC_SCHEDULE_REPLACE = 3;
