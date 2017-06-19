import grpc 
import messages_pb2
import messages_pb2_grpc

channel = grpc.insecure_channel('localhost:50051')
stub = messages_pb2_grpc.VinsteonRPCStub(channel)
feature = stub.SendCmd( messages_pb2.CmdMsg())
