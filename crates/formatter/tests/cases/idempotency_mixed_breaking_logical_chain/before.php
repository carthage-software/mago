<?php

class Library
{
    public function isVirtualHostableS3Bucket($bucketName, $allowSubdomains)
    {
        if ((is_null($bucketName)
            || (strlen($bucketName) < 3 || strlen($bucketName) > 63))
            || preg_match(self::IPV4_RE, $bucketName)
            || strtolower($bucketName) !== $bucketName
        ) {
            return false;
        }

        return true;
    }
}
