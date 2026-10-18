<?php

outer(inner($firstLongArgumentName, $secondLongArgumentName, $thirdLongArgumentName));

outer(new Notification(firstArgument: $firstLongArgumentName, secondArgument: $secondLongArgumentName));

outer(['first' => $firstLongArgumentName, 'second' => $secondLongArgumentName], $short);

outer($short, ['first' => $firstLongArgumentName, 'second' => $secondLongArgumentName]);

outer(['first' => $firstLongArgumentName, 'second' => $secondLongArgumentName]);

outer(function () { return first($firstLongArgumentName, $secondLongArgumentName); }, $short);

outer($short, function () { return first($firstLongArgumentName, $secondLongArgumentName); });

outer(fn ($item) => first($firstLongArgumentName, $secondLongArgumentName));

outer($short, fn ($item) => first($firstLongArgumentName, $secondLongArgumentName));

outer(match ($value) { 'first' => $firstLongArgumentName, default => $secondLongArgumentName });

outer(new class { public function run() { return first($firstLongArgumentName, $secondLongArgumentName); } });

outer(data: ['first' => $firstLongArgumentName, 'second' => $secondLongArgumentName]);

outer($short, inner($firstLongArgumentName, $secondLongArgumentName));
outer(inner($firstLongArgumentName, $secondLongArgumentName), $short);
outer(new Example(new Other($firstLongArgumentName, $secondLongArgumentName)));
outer($short, new Notification(first: $firstLongArgumentName, second: $secondLongArgumentName));
outer($short, named: ['first' => $firstLongArgumentName, 'second' => $secondLongArgumentName]);
outer(
    // Keep the leading comment.
    inner($firstLongArgumentName, $secondLongArgumentName),
);
outer(inner(...));
outer($firstLongArgumentName . $secondLongArgumentName);
#[Example(inner($firstLongArgumentName, $secondLongArgumentName))]
class Attributed {}

pauseAction($action, ['starts_on' => '2026-10-04', 'ends_on' => null])
    ->assertOk()
    ->assertJsonPath('data.action.pause.open_ended', true)
    ->assertJsonPath('data.action.pause.ends_on', null)
    ->assertJsonPath('data.action.pause.auto_ends_on', '2027-01-01');
