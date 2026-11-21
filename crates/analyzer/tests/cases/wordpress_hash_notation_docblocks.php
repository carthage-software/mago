<?php

declare(strict_types=1);

final class WordPressHashNotationPartial
{
    /**
     * Parsed ID.
     *
     * @var array {
     *     @type string $base ID base.
     *     @type array  $keys Keys for multidimensional.
     * }
     */
    public array $id_data = [];

    /**
     * @param int        $w     Width.
     * @param bool|array $crop  {
     *     Optional. Image cropping behavior.
     *
     *     @type string $0 The x crop position.
     *     @type string $1 The y crop position.
     * }
     * @return array {
     *     User counts.
     *
     *     @type int   $total_users Total number of users on the site.
     *     @type int[] $avail_roles Array of user counts keyed by user role.
     * }
     */
    public function resize(int $w, bool|array $crop = false): array
    {
        return ['total_users' => $w, 'avail_roles' => $crop === false ? [] : [1]];
    }

    /**
     * @return array{a: int}
     */
    public function shape(): array
    {
        return ['a' => 1];
    }

    /**
     * @return array {
     *     a: int,
     * }
     */
    public function spaced_shape(): array
    {
        return ['a' => 1];
    }
}
