<?php

final class Issue156
{
    private function test(): Response
    {
        \array_map([$this, 'addSql'], Vec\map($tables, static function ($table) {
            return <<<SQL
                update amount as a set posted_at = t.posted_at
                  from entry e, {$table} t
                 where e.id = a.entry_id
                   and e.id = t.entry_id;
                SQL;
        }));
    }
}
