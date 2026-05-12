<?php

class WrapTestCommand
{
	public function handle(): void
	{
		if (true) {
			if (true) {
				if (true) {
					if (true) {
						$someVeryLongOperation = 123_456_789 + 123_456_789 + 123_456_789 + 123_456_789 + 123_456_789;
					}
				}
			}
		}
	}

	public function veryLongFunctionSignature(int $param1, int $param2, int $param3, int $param4): void
	{
		//
	}
}
