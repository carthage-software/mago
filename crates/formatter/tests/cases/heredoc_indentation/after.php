<?php

$foo = <<<HEREDOC
    Foo
    Baz
    Bar
    HEREDOC;

function example()
{
    $sql = <<<SQL
        SELECT * FROM users
        WHERE id = 1
        SQL;

    $text = <<<TEXT
        Hello
        World
        TEXT;
}

class Foo
{
    public function bar()
    {
        $html = <<<HTML
            <div>
                <p>Hello</p>
            </div>
            HTML;
    }
}
