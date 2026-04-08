<?php

declare(strict_types=1);

/** @mago-expect lint:single-class-per-file */
namespace modules\ufml;

class tagDefinition
{
    public function __construct(
        public string $tag,
    ) {}
}

class tagList
{
    /** @var array<string,tagDefinition> */
    public array $list = [];

    public function __construct()
    {
        $this->append('root');
        $this->append('text');
    }

    public function append(string $tag): void
    {
        $this->list[$tag] = new tagDefinition($tag);
    }

    public function find(string $tag): ?tagDefinition
    {
        return $this->list[$tag] ?? null;
    }
}

class node
{
    /** @var node[] */
    public array $children = [];

    public int $object_id = 0;

    public function __construct(
        public string $tag,
        private readonly ?node $parent,
        public tagDefinition $tagdef,
    ) {}

    public function parent(): ?\modules\ufml\node
    {
        return $this->parent;
    }

    public function lastchild(): ?\modules\ufml\node
    {
        $n = count($this->children);
        if (!$n) {
            return null;
        }

        return $this->children[$n - 1];
    }
}

class Parser
{
    /**
     * @var \modules\ufml\tagList
     */
    public object $tags;

    public function __construct()
    {
        $this->tags = new tagList();
    }

    /**
     * @param array<int, array{bool,string}> $lexed
     */
    public function totree(array $lexed): node
    {
        $td = $this->tags->find('root');
        assert($td !== null, 'ECANTHAPPEN: no root tag found');
        $root = new node('root', null, $td);
        $cur = $root;
        $countlexed = count($lexed);
        for ($i = 0; $i < $countlexed; ++$i) {
            $entry = $lexed[$i];
            $opencloseflag = $entry[0];
            $tag = $entry[1];

            $td = $this->tags->find($tag);
            assert($td !== null, "ECANTHAPPEN: not tag {$tag} found");

            if ($cur === null) {
                $cur = $root;
            }

            if (3 == $opencloseflag) {
                $cur->children[] = new node($tag, $cur, $td);

                continue;
            }
            $look = $cur;
            while (true) {
                $look = $look->parent();
                if ($look === null || $look === $root) {
                    break;
                }
                if ($look->tag === $tag) {
                    break;
                }
            }
            if ($look === null || $look === $root) {
                continue;
            }
            $cur = $look;
            $cur = $cur->parent();
        }

        return $root;
    }
}
