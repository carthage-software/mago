<?php

namespace Database {
    /**
     * @phpstan-type Params = array{
     *     application_name?: string,
     *     charset?: string,
     *     dbname?: string,
     *     default_dbname?: string,
     *     host?: string,
     *     keepSlave?: bool,
     *     keepReplica?: bool,
     *     memory?: bool,
     *     password?: string,
     *     path?: string,
     *     persistent?: bool,
     *     port?: int,
     *     serverVersion?: string,
     *     url?: string,
     *     user?: string,
     *     unix_socket?: string,
     * }
     */
    final readonly class Connector
    {
        /**
         * @param Params $parameters
         */
        public static function connect(array $parameters): string
        {
            return 'Connected';
        }
    }
}

namespace Application {
    function connect(): string
    {
        return \Database\Connector::connect([
            'host' => 'localhost',
            'user' => 'root',
            'password' => 'password',
            'dbname' => 'test_db',
        ]);
    }
}
