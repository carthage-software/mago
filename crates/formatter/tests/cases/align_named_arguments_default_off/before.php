<?php

final class PlaygroundExamples
{
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
}
