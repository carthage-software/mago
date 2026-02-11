<?php

final class BlackfireProbe
{
    /**
     * @return BlackfireProbe
     */
    public static function getMainInstance() {}

    /**
     * @return bool
     */
    public static function isEnabled() {}

    /**
     * @param string $markerName
     */
    public static function addMarker($markerName = '') {}

    /**
     * @param string $query
     * @param string|null $envId
     * @param string|null $envToken
     * @param string|null $agentSocket
     */
    public function __construct($query, $envId = null, $envToken = null, $agentSocket = null) {}

    /**
     * @return bool
     */
    public function isVerified() {}

    /**
     * @param string $configuration
     */
    public function setConfiguration($configuration) {}

    /**
     * @return string
     */
    public function getResponseLine() {}

    /**
     * @return bool
     */
    public function enable() {}

    /**
     * @return bool
     */
    public function discard() {}

    /**
     * @return bool
     */
    public function disable() {}

    /**
     * @return bool
     */
    public function close() {}

    /**
     * @return string|null
     */
    public function createSubProfileQuery() {}

    /**
     * @return void
     */
    public static function setTransactionName(string $transactionName) {}

    /**
     * @return void
     */
    public static function ignoreTransaction() {}

    /**
     * @return void
     */
    public static function startTransaction() {}

    /**
     * @return void
     */
    public static function stopTransaction() {}
}
