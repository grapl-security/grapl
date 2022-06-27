# import grpc
#
# import graplinc
#
# from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2_grpc import AnalyzerServiceServicer
# from python_proto.graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.messages import RunAnalyzerRequest, \
#     RunAnalyzerResponse
#
#
# class AnalyzerServiceProto(AnalyzerServiceServicer):
#     def RunAnalyzer(
#             self,
#             request: graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2.RunAnalyzerRequest,
#             context: grpc.ServicerContext
#     ) -> graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2.RunAnalyzerResponse:
#         pass
#
#
# class AnalyzerService(object):
#     def __init__(self) -> None:
#         self._analyzer_service_proto = AnalyzerServiceProto()
#
#     def run_analyzer(self, request: RunAnalyzerRequest) -> RunAnalyzerResponse:
#         raise NotImplementedError()