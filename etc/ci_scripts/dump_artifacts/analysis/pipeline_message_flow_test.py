from ci_scripts.dump_artifacts.analysis.pipeline_message_flow import (
    get_num_received_messages,
)

TEST_INPUT_FROM_SYSMON_GENERATOR = """
{"timestamp":"2022-04-21T15:04:51.531623Z","level":"DEBUG","fields":{"message":"overriding region_name: \"us-east-1\""},"target":"grapl_config::env_helpers"}
{"timestamp":"2022-04-21T15:04:51.533053Z","level":"INFO","fields":{"message":"Starting process_loop"},"target":"graph_generator_lib"}
MONITORING|sysmon_generator|2022-04-21T15:05:11.560Z|sqs_executor.receive_message:20026|h|#success:true,empty_receive:true
{"timestamp":"2022-04-21T15:05:52.109084Z","level":"INFO","fields":{"message":"Received messages","message_batch_len":0},"target":"sqs_executor","span":{},"spans":[]}
MONITORING|sysmon_generator|2022-04-21T15:06:12.389Z|sqs_executor.receive_message:20028|h|#success:true,empty_receive:true
{"timestamp":"2022-04-21T15:06:12.389242Z","level":"INFO","fields":{"message":"Received messages","message_batch_len":3},"target":"sqs_executor","span":{},"spans":[]}
MONITORING|sysmon_generator|2022-04-21T15:06:13.155Z|sqs_executor.receive_message:515|h|#success:true,empty_receive:false
{"timestamp":"2022-04-21T15:06:13.155841Z","level":"INFO","fields":{"message":"Received messages","message_batch_len":1},"target":"sqs_executor","span":{},"spans":[]}
MONITORING|sysmon_generator|2022-04-21T15:06:13.156Z|redis_cache.all_exist.ms:0|h|#success:true
""".splitlines()


def test_receive_count_sysmon_generator() -> None:
    input = TEST_INPUT_FROM_SYSMON_GENERATOR
    receive_count = get_num_received_messages(input)
    assert receive_count == 4
