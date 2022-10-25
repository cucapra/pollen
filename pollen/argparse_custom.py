import argparse

class store_const_and_arg(argparse.Action):
    """                                                                     
    An argparse action which stores a constant and stores the argument(s)    
    passed to this flag. Useful when using flags as mutually exclusive
    switches that also need to accept an argument.                           
    """
    def __init__(self, option_strings, dest, nargs=None, **kwargs):
        if dest==None:
            raise Exception('dest must be defined')
        self.const_dest = dest

        # Convert an option string to a valid attribute name
        # Assume that no option string contains extra leading '-' chars       
        gen_dest = lambda s: s.lstrip('-').replace('-', '_')

        # Generate arg_dest according to the argparse rules for dest
        arg_dest = gen_dest(option_strings[0])
        if len(arg_dest) == 1 and len(option_strings) > 1:
            # Assume that no more than one short option is supplied
            arg_dest = gen_dest(option_strings[1])
        self.arg_dest = arg_dest
        
        super().__init__(option_strings, dest, **kwargs)

        
    def __call__(self, parser, namespace, values, option_string):
        setattr(namespace, self.const_dest, self.const)
        setattr(namespace, self.arg_dest, values)
        
