package main

import (
	"./messages"
	"encoding/binary"
	"golang.org/x/net/context"
	"google.golang.org/grpc"
	"log"
	"os"
	"strconv"
	"time"
)

func toUint32(target string) uint32 {
	result, err := strconv.ParseUint(target, 10, 32)
	if err != nil {
		log.Fatalf("unable to convert %s to an integer: %s" ,result , err)
	}
	return uint32(result)
}

func toByte(target string) uint8 {
	result, err := strconv.ParseUint(target, 16, 8)
	if err != nil {
		log.Fatalf("unable to convert %s to a byte: %s" ,result , err)
	}
	return uint8(result)
}

func connectToController(hostname string) *grpc.ClientConn {
	conn, err := grpc.Dial(hostname + ":50051", grpc.WithInsecure())
	if err != nil {
		log.Fatalf("did not connect: %s", err)
	}
	return conn
}

func main() {
	hostname := os.Args[1];
	conn := connectToController(hostname)
	defer conn.Close()

	client := messages.NewVinsteonRPCClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	var addrSlice = []byte{0, toByte(os.Args[2]), toByte(os.Args[3]), toByte(os.Args[4])}

	data := binary.BigEndian.Uint32(addrSlice)
	level := toUint32(os.Args[5])
	lightCtl := messages.CmdMsg_LightControl{
		LightControl: &messages.LightControl{Device: data, Level: level}}
	msg := messages.CmdMsg{Cmd: &lightCtl}
	client.SendCmd(ctx, &msg)
}
