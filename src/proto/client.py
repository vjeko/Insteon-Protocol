import grpc 
import binascii
import sys
import messages_pb2
import messages_pb2_grpc
import array

#channel = grpc.insecure_channel('localhost:50051')
channel = grpc.insecure_channel('controller.local:50051')
stub = messages_pb2_grpc.VinsteonRPCStub(channel)

binary = bytes([0,
    int(sys.argv[1], 16),
    int(sys.argv[2], 16),
    int(sys.argv[3], 16)])

print(int.from_bytes(binary, byteorder='big'))

light = messages_pb2.LightControl(
        device = int.from_bytes(binary, byteorder='big'),
        level = int(sys.argv[4]))
msg = messages_pb2.CmdMsg(lightControl = light)
#feature = stub.SendCmd( msg )
feature = stub.SendCmdReliable( msg )
