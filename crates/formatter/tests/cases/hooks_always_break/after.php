<?php

final readonly class DatabaseConfig
{
    public string $dsn {
        get => sprintf('mysql:host=%s:%s;dbname=%s', $this->host, $this->port, $this->database);
    }

    public DatabaseDialect $dialect {
        get => DatabaseDialect::MYSQL;
    }

    public DatabaseDialect $dialect { // foo
        get => DatabaseDialect::MYSQL;
        // bar
    } // baz

    public DatabaseDialect $dialect { // foo
        get {
            $a = 1;
            $b = 2;

            return DatabaseDialect::MYSQL;
        }
        set($value) {
            $this->dialect = $value;
        }
        // bar
    } // baz

    public DatabaseDialect $dialect {
        get {
            return DatabaseDialect::MYSQL;
        }
        set($value) {
            $this->dialect = $value;
        }
    }

    public DatabaseDialect $dialect {}

    public DatabaseDialect $dialect { // foo
        // bar
    } // baz
}
