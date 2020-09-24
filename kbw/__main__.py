import socket
import argparse
from random import randint
from os.path import dirname
from math import ceil
from enum import Enum
from struct import pack, unpack
from .kbw import kbw, set_plugin_path, set_seed 

ACK = pack('<b', 0)

class Command(Enum):
    EXIT = 0
    GET = 1
    DUMP = 2

def wait_command(client, quantum_execution): 
    while True:
        print('\tWaiting command...')

        command = Command(*unpack('<b', client.recv(1)))
        client.sendall(ACK)
        print('\t\tProcessing', command, sep='\t')

        if command == Command.GET:
            idx, = unpack('<Q', client.recv(8))
            result = quantum_execution.get_result(idx)

            print('\t\tSending result', idx, result, sep='\t')
            client.sendall(pack('<Q', result))

        elif command == Command.DUMP:
            idx, = unpack('<Q', client.recv(8))
            result = quantum_execution.get_dump_states(idx)

            print('\t\tSending dump', idx, result, sep='\t')
            client.sendall(pack('<Q',  len(result)))
            
            client.recv(1)

            for i in result:
                amplitude = quantum_execution.get_dump_amplitude(idx, i)
                client.sendall(pack('<QQ', i, len(amplitude)))
                client.recv(1)                
                
                print('\t\t\tSending amplitude', i, amplitude, sep='\t')

                for a in amplitude:
                    cx = pack('<dd', a.real, a.imag)
                    client.sendall(cx)
                    client.recv(1)
            
        elif command == Command.EXIT:
            break

    client.close()
    print("Connection closed\n")

def main():
    description = 'Ket Biswise Simulator server'
    print(description)
    print('============================\n')
    
    parser_args = argparse.ArgumentParser(prog='kbw', description=description)
    parser_args.add_argument('-p', metavar='4242', type=int, default=4242, help='Server port')
    parser_args.add_argument('-s', metavar='random', type=int, default=randint(0, 2048), help='Seed for the PRNG')
    parser_args.add_argument('-l', metavar='', type=str, help='Extra plugin path')
    args = parser_args.parse_args() 

    print('Setting up:') 
    buffer_size = 1024

    set_seed(args.s)
    print('\tSeed\t', args.s, sep='\t')

    plugin_path = dirname(__file__)
    if args.l:
        plugin_path += ':' + args.l
    set_plugin_path(plugin_path)
    print('\tPlugin PATH', plugin_path, sep='\t')

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind((socket.gethostbyname(socket.gethostname()), args.p))
    print('\tBind\t\t', socket.gethostbyname(socket.gethostname()), ':', args.p, sep='')

    print()

    server.listen(1)

    while True:
        print('Waiting connection...')

        client, address = server.accept()
        print('\tConnected by', address, sep='\t')

        print('\tWaiting KQASM...')
        
        file_size, =  unpack('<I', client.recv(4))
        client.sendall(ACK)
        print('\tKQASM size\t', file_size, sep='\t')
        
        kqasm_buffer = bytearray()
        for _ in range(ceil(file_size/buffer_size)):
            data = client.recv(buffer_size)
            kqasm_buffer += data
        
        kqasm_file = kqasm_buffer.decode()

        print('\tRunning KQASM...', end='\t')
        quantum_execution = kbw(kqasm_file)
        quantum_execution.run()

        client.sendall(ACK)
        print('done')
        
        wait_command(client, quantum_execution)

if __name__ == "__main__":
    main() 