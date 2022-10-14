'''
Combines the commandline interface for calyx_depth.py and parse_data.py. Run ./main.py -h for more info.
'''

import argparse
import json
import os.path
import subprocess
import tempfile
import warnings

from . import calyx_depth as depth
from . import parse_data

def config_parser(parser):

    depth.config_parser(parser)    

    parser.add_argument(
        '-a',
        '--auto-size',
        nargs='?',
        const='d',
        help='Provide an odgi file that will be used to calculate the hardware dimensions. If the flag is set with no argument, the argument of --file is used instead. Specified hardware dimensions take precedence.'
    )

    parser.set_defaults(action='gen')
    parser.add_argument(
        '-g',
        '--gen',
        dest='action',
        action='store_const',
        const='gen',
        help='Generate an accelerator. Should not be used with --run or --parse-data.'
    )
    parser.add_argument(
        '-r',
        '--run',
        dest='action',
        action='store_const',
        const='run',
        default='gen',
        help='Run node depth on the .og or .data --file. Outputs the node depth table. Should not be used with --gen or --parser-data.'
    )
    parser.add_argument(
        '-d',
        '--parse-data',
        dest='action',
        action='store_const',
        const='parse',
        default='gen',
        help='Parse the .og --file to accelerator input. Should not be used with --gen or --run.'
    )
    
    parser.add_argument(
        '-f',
        '--file',
        dest='filename',
        help='A .og or .data file. If --action=parse, this must be an odgi file.'
    )
    parser.add_argument(
        '-s',
        '--subset-paths',
        help='Should only be set if the action is not gen. Specifies a\
 subset of paths whose node depth to compute.'
    )

    parser.add_argument(
        '-x',
        '--accelerator',
        help='Specify a node depth accelerator to run. Should only be set if action is run.'
    )
    parser.add_argument(
        '--pr',
        action='store_true',
        help='Print profiling info. Passes the -pr flag to fud if --run is set.'
    )

    parser.add_argument(
        '--tmp-dir',
        help='Specify a directory to store temporary files in. The files will not be deleted at the end of execution.'
    )

def run_accel(args, tmp_dir_name):
    """
    Run the node depth accelerator
    """

    # Data parser
    parser = argparse.ArgumentParser()
    parse_data.config_parser(parser) 

    
    # Parse the data file if necessary
    out_file = args.out
    basename = os.path.basename(args.filename)
    base, ext = os.path.splitext(basename)

    if ext == '.data':
        if args.auto_size == 'd':
            warnings.warn('Cannot infer dimensions from .data file.',
                          SyntaxWarning)
        data_file = args.filename
    else:
        data_file = f'{tmp_dir_name}/{base}.data'
        new_args = [args.filename, '--out', data_file]
        parser.parse_args(new_args, namespace=args)
        parse_data.run(args)
        

    # Generate the accelerator if necessary
    if args.accelerator:
        futil_file = args.accelerator
    else:
        futil_file = f'{tmp_dir_name}/{base}.futil'
        new_args = [args.filename, '--out', futil_file]
        if args.auto_size == 'd':
            new_args.extend(['-a', args.filename])
        parser.parse_args(new_args, namespace=args)
        depth.run(args)

        
    # Compute the node depth
    cmd = ['fud', 'e', futil_file, '--to', 'interpreter-out',
           '-s', 'verilog.data', data_file]
    if args.pr:
        cmd.append('-pr')
        calyx_out = subprocess.run(cmd, capture_output=True, text=True)
        output = calyx_out.stdout

    else:
        calyx_out = subprocess.run(cmd, capture_output=True, text=True)
        # Convert calyx output to a node depth table
        calyx_out = json.loads(calyx_out.stdout)
        output = parse_data.from_calyx(calyx_out, True) # ndt

    # Output the ndt
    if out_file:
        with open(out_file, 'w') as out_file:
            out_file.write(output)
    else:
        print(output)
            

def run(args):
    
    if args.action == 'gen': # Generate an accelerator
        if args.filename or args.subset_paths or args.accelerator or args.pr:
            warnings.warn('--file, --subset-paths, --accelerator, and --pr will be ignored if action is gen.', SyntaxWarning)
        if args.auto_size == 'd':
            raise Exception('When action is gen, -a <file> must be specified.')
        
        depth.run(args)

    # Action is run or parse
    elif not args.filename:
        raise Exception('--file must be provided when action is parse or run.')
    
    elif args.action == 'parse': # Generate a data file
        if args.accelerator or args.pr:
            warnings.warn('--accelerator and --pr will be ignored if action is not run.', SyntaxWarning)

        parser = argparse.ArgumentParser()
        parse_data.config_parser(parser)
        parser.parse_args([args.filename], namespace=args) # Set defaults for all arguments
        parse_data.run(args)
        
    elif args.action == 'run': # Run the accelerator

        if args.tmp_dir:
            with open(args.tmp_dir, 'w') as tmp_dir_name:
                run_accel(args, tmp_dir_name)
        else:
            with tempfile.TemporaryDirectory() as tmp_dir_name:
#                print(os.path.isdir(tmp_dir_name))
                run_accel(args, tmp_dir_name)
            
        
def main():
    parser = argparse.ArgumentParser(conflict_handler='resolve')
    
    config_parser(parser)

    args = parser.parse_args()
    run(args)

    
if __name__ == '__main__':
    main()
