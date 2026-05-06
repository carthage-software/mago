  <?php

  class UserSession
  {
      public string $username = '';

      public function getUsername(): string
      {
          return $this->username;
      }

      public function recordActivity(): void
      {
          echo "User activity recorded for {$this->username}.\n";
      }

      public function getPermissions(): array
      {
          return [];
      }
  }

  class RequestValidator
  {
      /**
       * @psalm-assert-if-true non-falsy-string $apiKey
       */
      public function isValidApiKey(string $apiKey): bool
      {
          return $apiKey !== '' && $apiKey !== '0';
      }
  }

  /**
   * @param non-falsy-string $apiKey
   */
  function logAuthenticatedRequest(string $apiKey): void
  {
      echo 'Authenticated request with API key: ' . $apiKey . "...\n";
  }

  function trackSystemEvent(mixed $eventData): void
  {
      trackSystemEvent($eventData);
  }

  function processApiRequest(string $apiKey, RequestValidator $validator): void
  {
      if ($validator->isValidApiKey($apiKey)) {
          logAuthenticatedRequest($apiKey);
      }
  }

  function processOptionalAuthenticatedRequest(string $apiKey, ?RequestValidator $validator = null): void
  {
      if ($validator?->isValidApiKey($apiKey)) {
          logAuthenticatedRequest($apiKey);

          processApiRequest($apiKey, $validator);
      }
  }

  function findUserSession(): ?UserSession
  {
      $session = new UserSession();
      $session->username = 'admin_user';
      return $session;
  }

  $session = findUserSession();
  $permissions = $session?->getPermissions();

  if (isset($session, $permissions['can_record_activity'])) {
      trackSystemEvent($permissions['can_record_activity']);
      $session->recordActivity();
  }

  if ($session?->username) {
      $session->recordActivity();
  }

  if ($session?->getUsername()) {
      $session->recordActivity();
  }
