import grpc 
import messages_pb2
import messages_pb2_grpc

channel = grpc.insecure_channel('localhost:50051')
stub = messages_pb2_grpc.VinsteonRPCStub(channel)
light = messages_pb2.LightControl(device = 'Kitchen', level = 10)
msg = messages_pb2.CmdMsg(lightControl = light)
feature = stub.SendCmd( msg )
