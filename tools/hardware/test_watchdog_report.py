import unittest
from watchdog_report import analyze


def capture(case="sender-exit", fault="SenderExited", reboot=4.1):
    lines = [
        f"@0.000000 WATCHDOG PROBE case={case} reset=Some(ChipUsbUart); RS485=disabled",
        f"@0.100000 WATCHDOG ARMED case={case} reset_ms=1000 fault_after_ms=3000",
        f"@3.000000 WATCHDOG INJECT case={case} worker_progress=120",
    ]
    if fault:
        lines.append(f"@3.010000 WATCHDOG TRIPPED case={case} fault={fault}")
    lines.append(f"@{reboot-0.15:.6f} rst:0x8 (TG1WDT_SYS_RST),boot:0x8 (SPI_FAST_FLASH_BOOT)")
    lines.append(f"@{reboot:.6f} WATCHDOG PROBE case={case} reset=Some(CoreMwdt1); RS485=disabled")
    lines.extend(
        f"@{reboot+n:.6f} WATCHDOG RECOVERED case={case} stable_seconds={n}; RS485=disabled"
        for n in range(1, 13)
    )
    return "\n".join(lines)


class WatchdogReportTests(unittest.TestCase):
    def test_all_fault_classes_require_their_own_evidence(self):
        for case, fault, reboot in [
            ("sender-exit", "SenderExited", 4.1),
            ("sender-stall", None, 4.1),
            ("worker-stall", "Deadline", 6.1),
            ("worker-failed", "Worker", 4.1),
        ]:
            self.assertEqual(analyze(capture(case, fault, reboot), case)["reset"], "CoreMwdt1")

    def test_wrong_reset_incomplete_capture_and_missing_progress_are_rejected(self):
        original = capture()
        for text in [
            original.replace("CoreMwdt1", "ChipUsbUart"),
            original.replace("TG1WDT_SYS_RST", "USB_UART_CHIP_RESET"),
            original.replace("worker_progress=120", "worker_progress=0"),
            "\n".join(original.splitlines()[1:]),
            "\n".join(original.splitlines()[:-1]),
            original.replace("fault=SenderExited", "fault=Deadline"),
            original.replace("reset_ms=1000", "reset_ms=3000"),
            original.replace("case=sender-exit", "case=worker-stall", 1),
        ]:
            with self.subTest(text=text), self.assertRaises(AssertionError):
                analyze(text, "sender-exit")

    def test_slow_recovery_and_repeated_reset_are_rejected(self):
        with self.assertRaises(AssertionError):
            analyze(capture(reboot=7), "sender-exit")
        with self.assertRaises(AssertionError):
            analyze(capture() + "\n@20.000000 WATCHDOG PROBE case=sender-exit reset=Some(CoreMwdt1); RS485=disabled", "sender-exit")


if __name__ == "__main__":
    unittest.main()
