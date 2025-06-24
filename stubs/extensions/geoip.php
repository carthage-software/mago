<?php

/**
 * @param int $database
 *
 * @return string|null
 *
 * @pure
 */
function geoip_database_info($database = GEOIP_COUNTRY_EDITION)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 *
 * @pure
 */
function geoip_country_code_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 *
 * @pure
 */
function geoip_country_code3_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 *
 * @pure
 */
function geoip_country_name_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 *
 * @pure
 */
function geoip_continent_code_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 *
 * @pure
 */
function geoip_org_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return array|false
 *
 * @pure
 */
function geoip_record_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return int
 *
 * @pure
 */
function geoip_id_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return array|false
 *
 * @pure
 */
function geoip_region_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 *
 * @pure
 */
function geoip_isp_by_name($hostname)
{
}

/**
 * @param int $database
 *
 * @return bool|null
 *
 * @pure
 */
function geoip_db_avail($database)
{
}

/**
 * @return array
 *
 * @pure
 */
function geoip_db_get_all_info()
{
}

/**
 * @param int $database
 *
 * @return string|null
 *
 * @pure
 */
function geoip_db_filename($database)
{
}

/**
 * @param string $country_code
 *
 * @param string $region_code
 *
 * @return string|false
 *
 * @pure
 */
function geoip_region_name_by_code($country_code, $region_code)
{
}

/**
 * @param string $country_code
 * @param string $region_code
 *
 * @return string|false
 *
 * @pure
 */
function geoip_time_zone_by_country_and_region($country_code, $region_code = null)
{
}

const GEOIP_COUNTRY_EDITION = 1;

const GEOIP_REGION_EDITION_REV0 = 7;

const GEOIP_CITY_EDITION_REV0 = 6;

const GEOIP_ORG_EDITION = 5;

const GEOIP_ISP_EDITION = 4;

const GEOIP_CITY_EDITION_REV1 = 2;

const GEOIP_REGION_EDITION_REV1 = 3;

const GEOIP_PROXY_EDITION = 8;

const GEOIP_ASNUM_EDITION = 9;

const GEOIP_NETSPEED_EDITION = 10;

const GEOIP_DOMAIN_EDITION = 11;

const GEOIP_UNKNOWN_SPEED = 0;

const GEOIP_DIALUP_SPEED = 1;

const GEOIP_CABLEDSL_SPEED = 2;

const GEOIP_CORPORATE_SPEED = 3;

/**
 * @param string $hostname
 *
 * @return string|false
 */
function geoip_asnum_by_name($hostname)
{
}

/**
 * @param string $hostname
 *
 * @return string|false
 */
function geoip_netspeedcell_by_name($hostname)
{
}

/**
 * @param string $path
 *
 * @return void
 */
function geoip_setup_custom_directory($path)
{
}
