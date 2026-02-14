<?php

final readonly class OrderPlacedWorkflow
{
    public function waitForFulfillment(InFulfillment $event): void
    {
        $this->parcelService->createFromShipment($event->order->getShipmentEntityId());

        $dto->setMarket($this->marketConfig->getByType($dto->shippingMethodType->getMarketType()));
        $dto->setShippingZone($this->shippingZoneConfig->getByType($dto->shippingMethodType->getShippingZoneType()));

        if (true) {
            $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
            if (true) {
                $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                if (true) {
                    $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                    if (true) {
                        $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                        if (true) {
                            $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                            if (true) {
                                $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                                if (true) {
                                    $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                                    if (true) {
                                        $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                                        if (true) {
                                            $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                                            if (true) {
                                                $this->parcelService->createFromShipment($event->order->getShipmentEntityId());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
