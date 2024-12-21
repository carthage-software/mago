<?php

// Start of odbc v.1.0
use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;

/**
 * Toggle autocommit behaviour
 * @link https://php.net/manual/en/function.odbc-autocommit.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param bool $enable [optional] <p>
 * If <i>OnOff</i> is <b>TRUE</b>, auto-commit is enabled, if
 * it is <b>FALSE</b> auto-commit is disabled.
 * </p>
 * @return int|bool Without the <i>$enable</i> parameter, this function returns
 * auto-commit status for <i>connection_id</i>. Non-zero is
 * returned if auto-commit is on, 0 if it is off, or <b>FALSE</b> if an error
 * occurs.
 * <p>
 * If <i>$enable</i> is set, this function returns <b>TRUE</b> on
 * success and <b>FALSE</b> on failure.
 * </p>
 */
function odbc_autocommit($odbc, ?bool $enable = false): int|bool {}

/**
 * Handling of binary column data
 * @link https://php.net/manual/en/function.odbc-binmode.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * <p>
 * If <i>result_id</i> is 0, the
 * settings apply as default for new results.
 * Default for longreadlen is 4096 and
 * <i>mode</i> defaults to
 * ODBC_BINMODE_RETURN. Handling of binary long
 * columns is also affected by <b>odbc_longreadlen</b>.
 * </p>
 * @param int $mode <p>
 * Possible values for <i>mode</i> are:
 * <b>ODBC_BINMODE_PASSTHRU</b>: Passthru BINARY data
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function odbc_binmode($statement, int $mode): bool {}

/**
 * Close an ODBC connection
 * @link https://php.net/manual/en/function.odbc-close.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @return void No value is returned.
 */
function odbc_close($odbc): void {}

/**
 * Close all ODBC connections
 * @link https://php.net/manual/en/function.odbc-close-all.php
 * @return void No value is returned.
 */
function odbc_close_all(): void {}

/**
 * Lists the column names in specified tables
 * @link https://php.net/manual/en/function.odbc-columns.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog [optional] <p>
 * The qualifier.
 * </p>
 * @param string|null $schema [optional] <p>
 * The owner.
 * </p>
 * @param string|null $table [optional] <p>
 * The table name.
 * </p>
 * @param string|null $column [optional] <p>
 * The column name.
 * </p>
 * @return resource|false an ODBC result identifier or <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * TABLE_QUALIFIER
 * TABLE_SCHEM
 * TABLE_NAME
 * COLUMN_NAME
 * DATA_TYPE
 * TYPE_NAME
 * PRECISION
 * LENGTH
 * SCALE
 * RADIX
 * NULLABLE
 * REMARKS
 * </p>
 * <p>
 * The result set is ordered by TABLE_QUALIFIER, TABLE_SCHEM and
 * TABLE_NAME.
 * </p>
 */
function odbc_columns($odbc, ?string $catalog = null, ?string $schema = null, ?string $table = null, ?string $column = null) {}

/**
 * Commit an ODBC transaction
 * @link https://php.net/manual/en/function.odbc-commit.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function odbc_commit($odbc): bool {}

/**
 * Connect to a datasource
 * @link https://php.net/manual/en/function.odbc-connect.php
 * @param string $dsn <p>
 * The database source name for the connection. Alternatively, a
 * DSN-less connection string can be used.
 * </p>
 * @param string $user <p>
 * The username.
 * </p>
 * @param string $password <p>
 * The password.
 * </p>
 * @param int $cursor_option [optional] <p>
 * This sets the type of cursor to be used
 * for this connection. This parameter is not normally needed, but
 * can be useful for working around problems with some ODBC drivers.
 * </p>
 * <p>
 * The following constants are defined for cursortype:
 * SQL_CUR_USE_IF_NEEDED
 * </p>
 * @return resource|false an ODBC connection or (<b>FALSE</b>) on error.
 */
function odbc_connect(string $dsn, string $user, string $password, int $cursor_option = SQL_CUR_USE_DRIVER) {}

/**
 * Get cursorname
 * @link https://php.net/manual/en/function.odbc-cursor.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @return string|false the cursor name, as a string.
 */
function odbc_cursor($statement): string|false {}

/**
 * Returns information about a current connection
 * @link https://php.net/manual/en/function.odbc-data-source.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param int $fetch_type <p>
 * The <i>fetch_type</i> can be one of two constant types:
 * <b>SQL_FETCH_FIRST</b>, <b>SQL_FETCH_NEXT</b>.
 * Use <b>SQL_FETCH_FIRST</b> the first time this function is
 * called, thereafter use the <b>SQL_FETCH_NEXT</b>.
 * </p>
 * @return array|false|null <b>FALSE</b> on error, and an array upon success.
 */
#[ArrayShape(["server" => "string", "description" => "string"])]
function odbc_data_source($odbc, int $fetch_type): array|false|null {}

/**
 * Execute a prepared statement
 * @link https://php.net/manual/en/function.odbc-execute.php
 * @param resource $statement <p>
 * The result id resource, from <b>odbc_prepare</b>.
 * </p>
 * @param array $params [optional] <p>
 * Parameters in <i>parameter_array</i> will be
 * substituted for placeholders in the prepared statement in order.
 * Elements of this array will be converted to strings by calling this
 * function.
 * </p>
 * <p>
 * Any parameters in <i>parameter_array</i> which
 * start and end with single quotes will be taken as the name of a
 * file to read and send to the database server as the data for the
 * appropriate placeholder.
 * </p>
 * If you wish to store a string which actually begins and ends with
 * single quotes, you must add a space or other non-single-quote character
 * to the beginning or end of the parameter, which will prevent the
 * parameter from being taken as a file name. If this is not an option,
 * then you must use another mechanism to store the string, such as
 * executing the query directly with <b>odbc_exec</b>).
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function odbc_execute($statement, array $params = []): bool {}

/**
 * Get the last error code
 * @link https://php.net/manual/en/function.odbc-error.php
 * @param resource $odbc [optional] <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @return string If <i>$odbc</i> is specified, the last state
 * of that connection is returned, else the last state of any connection
 * is returned.
 * <p>
 * This function returns meaningful value only if last odbc query failed
 * (i.e. <b>odbc_exec</b> returned <b>FALSE</b>).
 * </p>
 */
function odbc_error($odbc = null): string {}

/**
 * Get the last error message
 * @link https://php.net/manual/en/function.odbc-errormsg.php
 * @param resource $odbc [optional] <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @return string If <i>$odbc</i> is specified, the last state
 * of that connection is returned, else the last state of any connection
 * is returned.
 * <p>
 * This function returns meaningful value only if last odbc query failed
 * (i.e. <b>odbc_exec</b> returned <b>FALSE</b>).
 * </p>
 */
function odbc_errormsg($odbc = null): string {}

/**
 * Prepare and execute an SQL statement
 * @link https://php.net/manual/en/function.odbc-exec.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string $query <p>
 * The SQL statement.
 * </p>
 * @param int $flags [optional] <p>
 * This parameter is currently not used.
 * </p>
 * @return resource|false an ODBC result identifier if the SQL command was executed
 * successfully, or <b>FALSE</b> on error.
 */
function odbc_exec($odbc, string $query, #[PhpStormStubsElementAvailable(from: '5.3', to: '7.4')] $flags = null) {}

/**
 * Fetch a result row as an associative array
 * @link https://php.net/manual/en/function.odbc-fetch-array.php
 * @param resource $statement <p>
 * The result resource from <b>odbc_exec</b>.
 * </p>
 * @param int $row [optional] <p>
 * Optionally choose which row number to retrieve.
 * </p>
 * @return array|false an array that corresponds to the fetched row, or <b>FALSE</b> if there
 * are no more rows.
 */
function odbc_fetch_array($statement, int $row = -1): array|false {}

/**
 * Fetch a result row as an object
 * @link https://php.net/manual/en/function.odbc-fetch-object.php
 * @param resource $statement <p>
 * The result resource from <b>odbc_exec</b>.
 * </p>
 * @param int $row [optional] <p>
 * Optionally choose which row number to retrieve.
 * </p>
 * @return stdClass|false an object that corresponds to the fetched row, or <b>FALSE</b> if there
 * are no more rows.
 */
function odbc_fetch_object($statement, int $row = -1): stdClass|false {}

/**
 * Fetch a row
 * @link https://php.net/manual/en/function.odbc-fetch-row.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param int|null $row [optional] <p>
 * If <i>row_number</i> is not specified,
 * <b>odbc_fetch_row</b> will try to fetch the next row in
 * the result set. Calls to <b>odbc_fetch_row</b> with and
 * without <i>row_number</i> can be mixed.
 * </p>
 * <p>
 * To step through the result more than once, you can call
 * <b>odbc_fetch_row</b> with
 * <i>row_number</i> 1, and then continue doing
 * <b>odbc_fetch_row</b> without
 * <i>row_number</i> to review the result. If a driver
 * doesn't support fetching rows by number, the
 * <i>row_number</i> parameter is ignored.
 * </p>
 * @return bool <b>TRUE</b> if there was a row, <b>FALSE</b> otherwise.
 */
function odbc_fetch_row($statement, ?int $row = null): bool {}

/**
 * Fetch one result row into array
 * @link https://php.net/manual/en/function.odbc-fetch-into.php
 * @param resource $statement <p>
 * The result resource.
 * </p>
 * @param array &$array <p>
 * The result array
 * that can be of any type since it will be converted to type
 * array. The array will contain the column values starting at array
 * index 0.
 * </p>
 * @param int $row [optional] <p>
 * The row number.
 * </p>
 * @return int|false the number of columns in the result;
 * <b>FALSE</b> on error.
 */
function odbc_fetch_into($statement, array &$array, int $row = 0): int|false {}

/**
 * Get the length (precision) of a field
 * @link https://php.net/manual/en/function.odbc-field-len.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param int $field <p>
 * The field number. Field numbering starts at 1.
 * </p>
 * @return int|false the field name as a string, or <b>FALSE</b> on error.
 */
function odbc_field_len($statement, int $field): int|false {}

/**
 * Get the scale of a field
 * @link https://php.net/manual/en/function.odbc-field-scale.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param int $field <p>
 * The field number. Field numbering starts at 1.
 * </p>
 * @return int|false the field scale as a integer, or <b>FALSE</b> on error.
 */
function odbc_field_scale($statement, int $field): int|false {}

/**
 * Get the columnname
 * @link https://php.net/manual/en/function.odbc-field-name.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param int $field <p>
 * The field number. Field numbering starts at 1.
 * </p>
 * @return string|false the field name as a string, or <b>FALSE</b> on error.
 */
function odbc_field_name($statement, int $field): string|false {}

/**
 * Datatype of a field
 * @link https://php.net/manual/en/function.odbc-field-type.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param int $field <p>
 * The field number. Field numbering starts at 1.
 * </p>
 * @return string|false the field type as a string, or <b>FALSE</b> on error.
 */
function odbc_field_type($statement, int $field): string|false {}

/**
 * Return column number
 * @link https://php.net/manual/en/function.odbc-field-num.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param string $field <p>
 * The field name.
 * </p>
 * @return int|false the field number as a integer, or <b>FALSE</b> on error.
 * Field numbering starts at 1.
 */
function odbc_field_num($statement, string $field): int|false {}

/**
 * Free resources associated with a result
 * @link https://php.net/manual/en/function.odbc-free-result.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @return bool Always returns <b>TRUE</b>.
 */
function odbc_free_result($statement): bool {}

/**
 * Retrieves information about data types supported by the data source
 * @link https://php.net/manual/en/function.odbc-gettypeinfo.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param int $data_type [optional] <p>
 * The data type, which can be used to restrict the information to a
 * single data type.
 * </p>
 * @return resource|false an ODBC result identifier or
 * <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * TYPE_NAME
 * DATA_TYPE
 * PRECISION
 * LITERAL_PREFIX
 * LITERAL_SUFFIX
 * CREATE_PARAMS
 * NULLABLE
 * CASE_SENSITIVE
 * SEARCHABLE
 * UNSIGNED_ATTRIBUTE
 * MONEY
 * AUTO_INCREMENT
 * LOCAL_TYPE_NAME
 * MINIMUM_SCALE
 * MAXIMUM_SCALE
 * </p>
 * <p>
 * The result set is ordered by DATA_TYPE and TYPE_NAME.
 * </p>
 */
function odbc_gettypeinfo($odbc, int $data_type = 0) {}

/**
 * Handling of LONG columns
 * @link https://php.net/manual/en/function.odbc-longreadlen.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param int $length <p>
 * The number of bytes returned to PHP is controlled by the parameter
 * length. If it is set to 0, Long column data is passed through to the
 * client.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function odbc_longreadlen($statement, int $length): bool {}

/**
 * Checks if multiple results are available
 * @link https://php.net/manual/en/function.odbc-next-result.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @return bool <b>TRUE</b> if there are more result sets, <b>FALSE</b> otherwise.
 */
function odbc_next_result($statement): bool {}

/**
 * Number of columns in a result
 * @link https://php.net/manual/en/function.odbc-num-fields.php
 * @param resource $statement <p>
 * The result identifier returned by <b>odbc_exec</b>.
 * </p>
 * @return int the number of fields, or -1 on error.
 */
function odbc_num_fields($statement): int {}

/**
 * Number of rows in a result
 * @link https://php.net/manual/en/function.odbc-num-rows.php
 * @param resource $statement <p>
 * The result identifier returned by <b>odbc_exec</b>.
 * </p>
 * @return int the number of rows in an ODBC result.
 * This function will return -1 on error.
 */
function odbc_num_rows($statement): int {}

/**
 * Open a persistent database connection
 * @link https://php.net/manual/en/function.odbc-pconnect.php
 * @param string $dsn
 * @param string $user
 * @param string $password
 * @param int $cursor_option [optional]
 * @return resource|false an ODBC connection id or 0 (<b>FALSE</b>) on
 * error.
 */
function odbc_pconnect(string $dsn, string $user, string $password, int $cursor_option = SQL_CUR_USE_DRIVER) {}

/**
 * Prepares a statement for execution
 * @link https://php.net/manual/en/function.odbc-prepare.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string $query <p>
 * The query string statement being prepared.
 * </p>
 * @return resource|false an ODBC result identifier if the SQL command was prepared
 * successfully. Returns <b>FALSE</b> on error.
 */
function odbc_prepare($odbc, string $query) {}

/**
 * Get result data
 * @link https://php.net/manual/en/function.odbc-result.php
 * @param resource $statement <p>
 * The ODBC resource.
 * </p>
 * @param string|int $field <p>
 * The field name being retrieved. It can either be an integer containing
 * the column number of the field you want; or it can be a string
 * containing the name of the field.
 * </p>
 * @return string|bool|null the string contents of the field, <b>FALSE</b> on error, <b>NULL</b> for
 * NULL data, or <b>TRUE</b> for binary data.
 */
function odbc_result($statement, string|int $field): string|bool|null {}

/**
 * Print result as HTML table
 * @link https://php.net/manual/en/function.odbc-result-all.php
 * @param resource $statement <p>
 * The result identifier.
 * </p>
 * @param string $format [optional] <p>
 * Additional overall table formatting.
 * </p>
 * @return int|false the number of rows in the result or <b>FALSE</b> on error.
 * @deprecated 8.1
 */
function odbc_result_all($statement, string $format = ''): int|false {}

/**
 * Rollback a transaction
 * @link https://php.net/manual/en/function.odbc-rollback.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function odbc_rollback($odbc): bool {}

/**
 * Adjust ODBC settings
 * @link https://php.net/manual/en/function.odbc-setoption.php
 * @param resource $odbc <p>
 * Is a connection id or result id on which to change the settings.
 * For SQLSetConnectOption(), this is a connection id.
 * For SQLSetStmtOption(), this is a result id.
 * </p>
 * @param int $which <p>
 * Is the ODBC function to use. The value should be
 * 1 for SQLSetConnectOption() and
 * 2 for SQLSetStmtOption().
 * </p>
 * @param int $option <p>
 * The option to set.
 * </p>
 * @param int $value <p>
 * The value for the given <i>option</i>.
 * </p>
 * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
 */
function odbc_setoption($odbc, int $which, int $option, int $value): bool {}

/**
 * Retrieves special columns
 * @link https://php.net/manual/en/function.odbc-specialcolumns.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param int $type When the type argument is <b>SQL_BEST_ROWID</b>,
 * <b>odbc_specialcolumns</b> returns the
 * column or columns that uniquely identify each row in the table.
 * When the type argument is <b>SQL_ROWVER</b>,
 * <b>odbc_specialcolumns</b> returns the column or columns in the
 * specified table, if any, that are automatically updated by the data source
 * when any value in the row is updated by any transaction.
 * @param string|null $catalog <p>
 * The qualifier.
 * </p>
 * @param string $schema <p>
 * The owner.
 * </p>
 * @param string $table <p>
 * The table.
 * </p>
 * @param int $scope <p>
 * The scope, which orders the result set.
 * </p>
 * @param int $nullable <p>
 * The nullable option.
 * </p>
 * @return resource|false an ODBC result identifier or <b>FALSE</b> on
 * failure.
 * <p>
 * The result set has the following columns:
 * SCOPE
 * COLUMN_NAME
 * DATA_TYPE
 * TYPE_NAME
 * PRECISION
 * LENGTH
 * SCALE
 * PSEUDO_COLUMN
 * </p>
 */
function odbc_specialcolumns($odbc, int $type, ?string $catalog, string $schema, string $table, int $scope, int $nullable) {}

/**
 * Retrieve statistics about a table
 * @link https://php.net/manual/en/function.odbc-statistics.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog <p>
 * The qualifier.
 * </p>
 * @param string $schema <p>
 * The owner.
 * </p>
 * @param string $table <p>
 * The table name.
 * </p>
 * @param int $unique <p>
 * The unique attribute.
 * </p>
 * @param int $accuracy <p>
 * The accuracy.
 * </p>
 * @return resource|false an ODBC result identifier or <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * TABLE_QUALIFIER
 * TABLE_OWNER
 * TABLE_NAME
 * NON_UNIQUE
 * INDEX_QUALIFIER
 * INDEX_NAME
 * TYPE
 * SEQ_IN_INDEX
 * COLUMN_NAME
 * COLLATION
 * CARDINALITY
 * PAGES
 * FILTER_CONDITION
 * </p>
 */
function odbc_statistics($odbc, ?string $catalog, string $schema, string $table, int $unique, int $accuracy) {}

/**
 * Get the list of table names stored in a specific data source
 * @link https://php.net/manual/en/function.odbc-tables.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog [optional] <p>
 * The qualifier.
 * </p>
 * @param string|null $schema [optional] <p>
 * The owner. Accepts search patterns ('%' to match zero or more
 * characters and '_' to match a single character).
 * </p>
 * @param string|null $table [optional] <p>
 * The name. Accepts search patterns ('%' to match zero or more
 * characters and '_' to match a single character).
 * </p>
 * @param string|null $types [optional] <p>
 * If <i>table_type</i> is not an empty string, it
 * must contain a list of comma-separated values for the types of
 * interest; each value may be enclosed in single quotes (') or
 * unquoted. For example, "'TABLE','VIEW'" or "TABLE, VIEW". If the
 * data source does not support a specified table type,
 * <b>odbc_tables</b> does not return any results for
 * that type.
 * </p>
 * @return resource|false an ODBC result identifier containing the information
 * or <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * TABLE_QUALIFIER
 * TABLE_OWNER
 * TABLE_NAME
 * TABLE_TYPE
 * REMARKS
 * </p>
 */
function odbc_tables($odbc, ?string $catalog = null, ?string $schema = null, ?string $table = null, ?string $types = null) {}

/**
 * Gets the primary keys for a table
 * @link https://php.net/manual/en/function.odbc-primarykeys.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog
 * @param string $schema
 * @param string $table
 * @return resource|false an ODBC result identifier or <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * TABLE_QUALIFIER
 * TABLE_OWNER
 * TABLE_NAME
 * COLUMN_NAME
 * KEY_SEQ
 * PK_NAME
 * </p>
 */
function odbc_primarykeys($odbc, ?string $catalog, string $schema, string $table) {}

/**
 * Lists columns and associated privileges for the given table
 * @link https://php.net/manual/en/function.odbc-columnprivileges.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog <p>
 * The qualifier.
 * </p>
 * @param string $schema <p>
 * The owner.
 * </p>
 * @param string $table <p>
 * The table name.
 * </p>
 * @param string $column <p>
 * The <i>column_name</i> argument accepts search
 * patterns ('%' to match zero or more characters and '_' to match a
 * single character).
 * </p>
 * @return resource|false an ODBC result identifier or <b>FALSE</b> on failure.
 * This result identifier can be used to fetch a list of columns and
 * associated privileges.
 * <p>
 * The result set has the following columns:
 * TABLE_QUALIFIER
 * TABLE_OWNER
 * TABLE_NAME
 * GRANTOR
 * GRANTEE
 * PRIVILEGE
 * IS_GRANTABLE
 * </p>
 * <p>
 * The result set is ordered by TABLE_QUALIFIER, TABLE_OWNER and
 * TABLE_NAME.
 * </p>
 */
function odbc_columnprivileges($odbc, ?string $catalog, string $schema, string $table, string $column) {}

/**
 * Lists tables and the privileges associated with each table
 * @link https://php.net/manual/en/function.odbc-tableprivileges.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog <p>
 * The qualifier.
 * </p>
 * @param string $schema <p>
 * The owner. Accepts the following search patterns:
 * ('%' to match zero or more characters and '_' to match a single character)
 * </p>
 * @param string $table <p>
 * The name. Accepts the following search patterns:
 * ('%' to match zero or more characters and '_' to match a single character)
 * </p>
 * @return resource|false An ODBC result identifier or <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * TABLE_QUALIFIER
 * TABLE_OWNER
 * TABLE_NAME
 * GRANTOR
 * GRANTEE
 * PRIVILEGE
 * IS_GRANTABLE
 * </p>
 */
function odbc_tableprivileges($odbc, ?string $catalog, string $schema, string $table) {}

/**
 * Retrieves a list of foreign keys
 * @link https://php.net/manual/en/function.odbc-foreignkeys.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $pk_catalog <p>
 * The primary key qualifier.
 * </p>
 * @param string $pk_schema <p>
 * The primary key owner.
 * </p>
 * @param string $pk_table <p>
 * The primary key table.
 * </p>
 * @param string $fk_catalog <p>
 * The foreign key qualifier.
 * </p>
 * @param string $fk_schema <p>
 * The foreign key owner.
 * </p>
 * @param string $fk_table <p>
 * The foreign key table.
 * </p>
 * @return resource|false an ODBC result identifier or <b>FALSE</b> on failure.
 * <p>
 * The result set has the following columns:
 * PKTABLE_QUALIFIER
 * PKTABLE_OWNER
 * PKTABLE_NAME
 * PKCOLUMN_NAME
 * FKTABLE_QUALIFIER
 * FKTABLE_OWNER
 * FKTABLE_NAME
 * FKCOLUMN_NAME
 * KEY_SEQ
 * UPDATE_RULE
 * DELETE_RULE
 * FK_NAME
 * PK_NAME
 * </p>
 * <p>
 * If <i>pk_table</i> contains a table name,
 * <b>odbc_foreignkeys</b> returns a result set
 * containing the primary key of the specified table and all of the
 * foreign keys that refer to it.
 * If <i>fk_table</i> contains a table name,
 * <b>odbc_foreignkeys</b> returns a result set
 * containing all of the foreign keys in the specified table and the
 * primary keys (in other tables) to which they refer.
 * If both <i>pk_table</i> and
 * <i>fk_table</i> contain table names,
 * <b>odbc_foreignkeys</b> returns the foreign keys in
 * the table specified in <i>fk_table</i> that refer
 * to the primary key of the table specified in
 * <i>pk_table</i>
 * </p>
 */
function odbc_foreignkeys($odbc, ?string $pk_catalog, string $pk_schema, string $pk_table, string $fk_catalog, string $fk_schema, string $fk_table) {}

/**
 * Get the list of procedures stored in a specific data source
 * @link https://php.net/manual/en/function.odbc-procedures.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog <p>The catalog ('qualifier' in ODBC 2 parlance).</p>
 * @param string|null $schema <p>The schema ('owner' in ODBC 2 parlance). This parameter accepts the
 *  following search patterns: % to match zero or more characters, and _ to match a single character.</p>
 * @param string|null $procedure <p>The proc. This parameter accepts the following search patterns:
 *  % to match zero or more characters, and _ to match a single character.</p>
 * @return resource|false <p>an ODBC
 * result identifier containing the information or <b>FALSE</b> on failure.
 * </p>
 * <p>
 * The result set has the following columns:
 * PROCEDURE_QUALIFIER
 * PROCEDURE_OWNER
 * PROCEDURE_NAME
 * NUM_INPUT_PARAMS
 * NUM_OUTPUT_PARAMS
 * NUM_RESULT_SETS
 * REMARKS
 * PROCEDURE_TYPE
 * </p>
 */
function odbc_procedures($odbc, ?string $catalog = null, ?string $schema = null, ?string $procedure = null) {}

/**
 * Retrieve information about parameters to procedures
 * @link https://php.net/manual/en/function.odbc-procedurecolumns.php
 * @param resource $odbc <p>The ODBC connection identifier,
 * see <b>odbc_connect</b> for details.</p>
 * @param string|null $catalog <p>The catalog ('qualifier' in ODBC 2 parlance).</p>
 * @param string|null $schema <p>The schema ('owner' in ODBC 2 parlance). This parameter accepts the
 * following search patterns: % to match zero or more characters, and _ to match a single character.</p>
 * @param string|null $procedure <p>The proc. This parameter accepts the following search patterns:
 * % to match zero or more characters, and _ to match a single character.</p>
 * @param string|null $column <p>The column. This parameter accepts the following search patterns:
 * % to match zero or more characters, and _ to match a single character.</p>
 * @return resource|false <p>the list of input and output parameters, as well as the
 * columns that make up the result set for the specified procedures.
 * Returns an ODBC result identifier or <b>FALSE</b> on failure.
 * </p>
 * <p>
 * The result set has the following columns:
 * PROCEDURE_QUALIFIER
 * PROCEDURE_OWNER
 * PROCEDURE_NAME
 * COLUMN_NAME
 * COLUMN_TYPE
 * DATA_TYPE
 * TYPE_NAME
 * PRECISION
 * LENGTH
 * SCALE
 * RADIX
 * NULLABLE
 * REMARKS
 * </p>
 */
function odbc_procedurecolumns($odbc, ?string $catalog = null, ?string $schema = null, ?string $procedure = null, ?string $column = null) {}

/**
 * Alias of <b>odbc_exec</b>
 * @link https://php.net/manual/en/function.odbc-do.php
 * @param $odbc
 * @param string $query
 */
function odbc_do($odbc, string $query) {}

/**
 * Alias of <b>odbc_field_len</b>
 * @link https://php.net/manual/en/function.odbc-field-precision.php
 * @param $statement
 * @param int $field
 */
function odbc_field_precision($statement, int $field): int|false {}

function odbc_connection_string_is_quoted(string $str): bool {}

function odbc_connection_string_should_quote(string $str): bool {}

function odbc_connection_string_quote(string $str): string {}

define('ODBC_TYPE', "unixODBC");
define('ODBC_BINMODE_PASSTHRU', 0);
define('ODBC_BINMODE_RETURN', 1);
define('ODBC_BINMODE_CONVERT', 2);
define('SQL_ODBC_CURSORS', 110);
define('SQL_CUR_USE_DRIVER', 2);
define('SQL_CUR_USE_IF_NEEDED', 0);
define('SQL_CUR_USE_ODBC', 1);
define('SQL_CONCURRENCY', 7);
define('SQL_CONCUR_READ_ONLY', 1);
define('SQL_CONCUR_LOCK', 2);
define('SQL_CONCUR_ROWVER', 3);
define('SQL_CONCUR_VALUES', 4);
define('SQL_CURSOR_TYPE', 6);
define('SQL_CURSOR_FORWARD_ONLY', 0);
define('SQL_CURSOR_KEYSET_DRIVEN', 1);
define('SQL_CURSOR_DYNAMIC', 2);
define('SQL_CURSOR_STATIC', 3);
define('SQL_KEYSET_SIZE', 8);
define('SQL_FETCH_FIRST', 2);
define('SQL_FETCH_NEXT', 1);
define('SQL_CHAR', 1);
define('SQL_VARCHAR', 12);
define('SQL_LONGVARCHAR', -1);
define('SQL_DECIMAL', 3);
define('SQL_NUMERIC', 2);
define('SQL_BIT', -7);
define('SQL_TINYINT', -6);
define('SQL_SMALLINT', 5);
define('SQL_INTEGER', 4);
define('SQL_BIGINT', -5);
define('SQL_REAL', 7);
define('SQL_FLOAT', 6);
define('SQL_DOUBLE', 8);
define('SQL_BINARY', -2);
define('SQL_VARBINARY', -3);
define('SQL_LONGVARBINARY', -4);
define('SQL_DATE', 9);
define('SQL_TIME', 10);
define('SQL_TIMESTAMP', 11);
define('SQL_TYPE_DATE', 91);
define('SQL_TYPE_TIME', 92);
define('SQL_TYPE_TIMESTAMP', 93);
define('SQL_WCHAR', -8);
define('SQL_WVARCHAR', -9);
define('SQL_WLONGVARCHAR', -10);
define('SQL_BEST_ROWID', 1);
define('SQL_ROWVER', 2);
define('SQL_SCOPE_CURROW', 0);
define('SQL_SCOPE_TRANSACTION', 1);
define('SQL_SCOPE_SESSION', 2);
define('SQL_NO_NULLS', 0);
define('SQL_NULLABLE', 1);
define('SQL_INDEX_UNIQUE', 0);
define('SQL_INDEX_ALL', 1);
define('SQL_ENSURE', 1);
define('SQL_QUICK', 0);

// End of odbc v.1.0
