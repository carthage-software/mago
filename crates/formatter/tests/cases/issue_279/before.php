<?php

            if (
                in_array(
                    $object->getVeryLongAndInterestingObjectType(),
                    SuperClass::VERY_LONG_AND_DESCRIPTIVE_CONSTANT_NAME,
                )
                && (
                    !$object->veryLongConditionCheckWithLotsOfText()
                    && !$object->evenLongerConditionCheckWithEvenMoreText()
                )
            ) {}

            if (false) {
                
                if (
                    in_array($object->getVeryLongAndInterestingObjectType(), SuperClass::VERY_LONG_AND_DESCRIPTIVE_CONSTANT_NAME) &&
                        (!$object->veryLongConditionCheckWithLotsOfText() && !$object->evenLongerConditionCheckWithEvenMoreText())
                ) {
                }

            }
            
            if (false) {
            if (false) {
                
                if (
                    in_array($object->getVeryLongAndInterestingObjectType(), SuperClass::VERY_LONG_AND_DESCRIPTIVE_CONSTANT_NAME) &&
                        (!$object->veryLongConditionCheckWithLotsOfText() && !$object->evenLongerConditionCheckWithEvenMoreText())
                ) {
                }

            }            }