from os import environ

import grapl_tests_common
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_tests_common.setup import AnalyzerUpload
from grapl_tests_common.sleep import verbose_sleep
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
            "/home/grapl/etc/sample_data/eventlog.xml",
        ),
    )
    grapl_tests_common.setup.setup(
        analyzers=analyzers,
        test_data=test_data,
    )

    grapl_tests_common.setup.exec_pytest()


if __name__ == "__main__":
    main()
