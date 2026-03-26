<?php

function getSql()
{
    return /** @lang PostgreSQL */ <<<SQL
            SELECT * FROM "table"
        SQL;
}
