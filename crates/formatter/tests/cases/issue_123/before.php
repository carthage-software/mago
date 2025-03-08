<?php

new UserPermission(
    user: $this,
    permission: $permission,
)->save();

$this->console->keyValue('Global cache', match ($this->cacheConfig->enable) {
    true => '<style="bold fg-green">ENABLED</style>',
    false => '<style="bold fg-red">FORCEFULLY DISABLED</style>',
    default => '<style="bold fg-gray">DISABLED</style>',
});

$this->pool = $this->cacheConfig->projectCachePool ?? new FilesystemAdapter(
    directory: path($this->cacheConfig->directory, 'project')->toString(),
);

$this->expectExceptionObject(
    new CommandHandlerNotFound($command),
);

