<?php

$queryBuilder
    ->leftJoin(sprintf('%s.hosts', $rootAlias), 'announcement_host_filter')
    ->andWhere($queryBuilder->expr()->orX(
        'announcement_host_filter.id = :announcementHostId',
        'announcement_host_filter.id IS NULL',
    ))
    ->setParameter('announcementHostId', $host->getId()?->toBinary())
;

$context
    ->buildViolation('Please provide a flat price when selling early check-in.')
    ->atPath('earlyCheckInFlatPrice')
    ->addViolation()
;

$this
    ->createQueryBuilder('cu')
    ->update()
    ->set('cu.escalations', 'cu.escalations + 1')
    ->where('cu.chat = :chat')
    ->setParameter('chat', $chat->getId())
    ->getQuery()
    ->execute()
;

$validatorFactory
    ->createForEntity($entity)
    ->withScenario('night-audit')
    ->withSeverity('warning')
    ->markAsSkipped()
;

$validatorFactory->createForEntity($entity)->withScenario('night-audit');

Network\SocketOptions::create()
    ->withAddressReuse(false)
    ->withPortReuse(false)
    ->withBroadcast(true)
;

$criteria = Criteria::create()
    ->where(Criteria::expr()->eq('data', (string) $id))
    ->setMaxResults(1)
;

if (null !== $language) {
    $criteria = $criteria
        ->andWhere(Criteria::expr()->eq('language', $language))
    ;
}

$soapOptions = ExtSoapOptions::defaults($wsdl)
    ->disableWsdlCache()
    ->withClassMap(PortugalSefClassmap::getCollection())
;

return parent::configureAssets()
    ->addCssFile('admin-typography.css')
    ->addCssFile('admin-primary.css')
    ->addCssFile('admin-tokens.css')
    ->addJsFile('build/tinymce/tinymce.min.js')
    ->addWebpackEncoreEntry(Asset::new('admin'))
;

$cache
    ->expects($this->once())
    ->method('get')
    ->willReturn("cache-entry:$token")
    ->withTag('notifications')
    ->withTtl(3600)
    ->withLock('cache-lock-key-for-current-user')
    ->send()
;

$cache
    ->expects($this->once())
    ->method('get')
    ->willReturn('cache-entry:$token')
    ->withTag('notifications')
    ->withTtl(3600)
    ->withLock('cache-lock-key-for-current-user')
    ->send()
;
