<?php

namespace MongoDB\BSON;

use stdClass;

/**
 * Classes may implement this interface to take advantage of automatic ODM (object document mapping) behavior in the driver.
 * @link https://php.net/manual/en/class.mongodb-bson-persistable.php
 */
interface Persistable extends Unserializable, Serializable
{
    /** @since 1.17.0 */
    public function bsonSerialize(): array|stdClass|Document;
}
