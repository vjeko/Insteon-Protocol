import grpc 
import sys
import messages_pb2
import messages_pb2_grpc
import array

channel = grpc.insecure_channel('localhost:50051')
stub = messages_pb2_grpc.VinsteonRPCStub(channel)

dev = [int(sys.argv[1], 16), int(sys.argv[2], 16), int(sys.argv[3], 16) ]
byte_array = array.array('B', dev).tostring()
light = messages_pb2.LightControl(
        device = byte_array,
        #device = b'\x41\x1D\x1E', 
        level = int(sys.argv[4]))
msg = messages_pb2.CmdMsg(lightControl = light)
feature = stub.SendCmd( msg )
