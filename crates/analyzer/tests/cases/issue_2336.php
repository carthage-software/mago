<?php

declare(strict_types=1);

/**
 * @psalm-type NodeA = array{
 *     kind: string,
 *     definition?: string|null,
 *     option?: string|null,
 *     updatable?: bool|null,
 * }
 * @psalm-type NodeB = array{
 *     position: int|string|null,
 *     default: string|int|bool|null,
 *     nullable: bool|null,
 *     type: string,
 *     max_length: int|string|null,
 *     octet_length: int|string|null,
 *     precision: int|string|null,
 *     scale: int|string|null,
 *     unsigned: bool|null,
 *     extra: array<string, mixed>,
 * }
 * @psalm-type NodeC = array{
 *     kind?: string,
 *     match?: string,
 *     on_update?: string,
 *     on_delete?: string,
 *     fields?: list<string>,
 *     target_namespace?: string,
 *     target_name?: string,
 *     target_fields?: list<string>,
 *     clause?: string,
 * }
 * @psalm-type NodeD = array{
 *     owner: string,
 *     name: string,
 *     field: string,
 *     position: int,
 * }
 * @psalm-type NodeE = array{
 *     name: string,
 *     on_update: string,
 *     on_delete: string,
 *     target_name: string,
 *     target_field: string,
 * }
 * @psalm-type NodeF = array{
 *     event: string,
 *     catalog: string,
 *     namespace: string,
 *     subject: string,
 *     order: string,
 *     condition: string|null,
 *     statement: string,
 *     orientation: string,
 *     timing: string,
 *     old_table: string|null,
 *     new_table: string|null,
 *     old_row: string,
 *     new_row: string,
 *     created: DateTimeInterface|null,
 * }
 * @psalm-type Tree = array{
 *     names?: list<string>,
 *     a?: array<string, array<string, NodeA>>,
 *     b?: array<string, array<string, array<string, NodeB>>>,
 *     c?: array<string, array<string, array<string, NodeC>>>,
 *     d?: array<string, list<NodeD>>,
 *     e?: array<string, list<NodeE>>,
 *     f?: array<string, array<string, NodeF>>,
 * }
 */
final class Repro
{
    /** @var Tree */
    private array $tree = [];

    public function walk(string ...$keys): void
    {
        $node = &$this->tree;
        foreach ($keys as $key) {
            $node = &$node[$key];
        }
    }

    public function walkWithInitialization(string ...$keys): void
    {
        $node = &$this->tree;
        foreach ($keys as $key) {
            if (!isset($node[$key])) {
                $node[$key] = [];
            }

            $node = &$node[$key];
        }
    }

    public function walkLocalCopy(string ...$keys): void
    {
        $tree = $this->tree;
        $node = &$tree;
        foreach ($keys as $key) {
            $node = &$node[$key];
        }
    }

    public function walkWithOtherMutation(string ...$keys): int
    {
        $node = &$this->tree;
        $count = 0;
        foreach ($keys as $key) {
            $node = &$node[$key];
            $count++;
        }

        return $count;
    }
}
