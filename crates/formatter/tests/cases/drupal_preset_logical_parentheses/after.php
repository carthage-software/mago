<?php

namespace Drupal\Core\Access;

class AccessCheck {

  public function isAllowed($account, $entity, $operation) {
    if ($account->isAnonymous() || ($entity->isPublished() && $operation === 'view')) {
      return TRUE;
    }

    if (
      $this->token === NULL
      || ($this->lookahead !== NULL
      && ($this->lookahead->position - $this->token->position) === strlen($this->token->value))
    ) {
      return FALSE;
    }

    return (
      $account->hasPermission('bypass access')
      || ($entity->getOwnerId() === $account->id() && $operation !== 'delete')
    );
  }

}
