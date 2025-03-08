<?php

return new CreateTableStatement('user_permissions')
  ->primary()
  ->belongsTo('user_permissions.user_id', 'users.id')
  ->belongsTo('user_permissions.permission_id', 'permissions.id');
  
$item = $this->getCachePool()
    ->getItem($key)
    ->set($value);

    $appNamespace = str($this->ask('Which namespace do you wish to use?', default: $defaultAppNamespace))
          ->trim('\\')
          ->append('\\')
          ->toString();  