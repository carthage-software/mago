<?php

declare(strict_types=1);

function inspect(SQLite3Stmt $statement): void
{
    $statement->reset();

    // @mago-expect analysis:unavailable-method
    $statement->setExplain(1);

    // @mago-expect analysis:unavailable-method
    $_ = $statement->explain();

    // @mago-expect analysis:unavailable-method
    $_ = $statement->busy();

    // @mago-expect analysis:unavailable-class-constant
    $_ = SQLite3Stmt::EXPLAIN_MODE_PREPARED;

    // @mago-expect analysis:unavailable-class-constant
    $_ = SQLite3Stmt::EXPLAIN_MODE_EXPLAIN;

    // @mago-expect analysis:unavailable-class-constant
    $_ = SQLite3Stmt::EXPLAIN_MODE_EXPLAIN_QUERY_PLAN;
}
