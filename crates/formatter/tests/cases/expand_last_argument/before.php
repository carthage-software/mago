<?php

$value = strtr($this->fileExcerpt($trace['file'], $trace['line'], 5), [
    '#DD0000' => 'var(--highlight-string)',
    '#007700' => 'var(--highlight-keyword)',
    '#0000BB' => 'var(--highlight-default)',
    '#FF8000' => 'var(--highlight-comment)',
]);

it('needs a proper payload', function () {
    post(route('mosquito-deployment.store'), [
        'artifactUuid' => str()->uuid()->toString(),
    ])->assertInvalid();
});

it('handles a proper payload', function () {
    $artifact = Artifact::factory()->create();

    post(route('mosquito-deployment.store'), [
        'artifactUuid' => $artifact->uuid,
    ])->assertSuccessful();
});