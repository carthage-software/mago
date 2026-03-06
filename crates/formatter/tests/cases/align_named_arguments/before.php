<?php

#[UniqueEntity(
    fields: ['host', 'source', 'externalId', 'userConfiguration'],
    message: 'A lock with this Integration lock ID already exists for this host/source.',
)]
#[ApiResource(
    normalizationContext: ['groups' => ['host:read']],
    denormalizationContext: ['groups' => ['host:write']],
    graphQlOperations: [
        new Query(),
        new QueryCollection(),
    ],
)]
final class PlaygroundExamples
{
    #[ORM\OneToOne(
        mappedBy: 'host',
        targetEntity: HostNotification::class,
        cascade: ['persist', 'remove'],
        fetch: 'EXTRA_LAZY',
    )]
    private ?HostNotification $notificationSettings = null;

    #[ORM\OneToOne(
        mappedBy: 'primaryUser',
        targetEntity: Organization::class,
        cascade: ['persist', 'remove'],
        fetch: 'EXTRA_LAZY',
        orphanRemoval: true,
    )]
    private ?Organization $organization = null;

    public static function bookingAvailability($bookingId, array $unitStatuses): self
    {
        return new self(
            bookingId: $bookingId,
            overallStatus: self::deriveOverallStatus($unitStatuses),
            unitStatuses: $unitStatuses,
        );
    }

    public static function unauthorized($sharingTime): self
    {
        return new self(
            status: LockCodeAvailabilityStatus::UNAUTHORIZED,
            message: 'For privacy and security reasons, access codes can only be shared with the primary guest on the reservation.',
            availableFrom: $sharingTime,
        );
    }

    public static function sell(
        array $availableSlots,
        array $pricing,
        string $regularCheckInTime,
        string $earliestCheckInTime,
    ): self {
        return new self(
            action: EarlyCheckInAction::SELL,
            availableSlots: $availableSlots,
            pricing: $pricing,
            regularCheckInTime: $regularCheckInTime,
            earliestCheckInTime: $earliestCheckInTime,
        );
    }

    public static function compact(): self
    {
        return new self(status: LockCodeAvailabilityStatus::AVAILABLE, codes: $codes);
    }

    public static function withLongAttributeCall(): object
    {
        return new ApiResource(
            normalizationContext: ['groups' => ['host:read']],
            denormalizationContext: ['groups' => ['host:write']],
            graphQlOperations: [
                new Query(),
                new QueryCollection(),
            ],
        );
    }
}
