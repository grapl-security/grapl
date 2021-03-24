from pathlib import Path

from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_tests_common import setup_tests
from grapl_tests_common.upload_analyzers import AnalyzerUpload
from grapl_tests_common.upload_test_data import UploadSysmonLogsTestData


def main() -> None:
    wait_for_vsc_debugger("grapl_e2e_tests")
    analyzers = (
        AnalyzerUpload(
            "/home/grapl/etc/local_grapl/suspicious_svchost/main.py",
            "analyzers/suspicious_svchost/main.py",
        ),
        AnalyzerUpload(
            "/home/grapl/etc/local_grapl/unique_cmd_parent/main.py",
            "analyzers/unique_cmd_parent/main.py",
        ),
    )

    test_data = (
        UploadSysmonLogsTestData(
            Path("/home/grapl/etc/sample_data/eventlog.xml").resolve()
        ),
    )
    setup_tests.setup_tests(
        analyzers=analyzers,
        test_data=test_data,
    )

    setup_tests.exec_pytest()


if __name__ == "__main__":
    main()
