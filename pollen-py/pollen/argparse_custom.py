import argparse

class store_const_and_arg(argparse.Action):
    """                                                                     
    An argparse action which stores a constant and stores the argument(s)    
    passed to this flag. Useful when using flags as mutually exclusive
    switches that also need to accept an argument.                           
    'dest' is the destination for option arguments (as usual), and
    'dest2' is the destination where 'const' is stored.
    """
    def __init__(self, option_strings, dest, dest2=None, nargs=None, **kwargs):
        if dest2==None:
            raise Exception('dest2 must be defined')
        self.const_dest = dest2
        
        super().__init__(option_strings, dest, **kwargs)

        
    def __call__(self, parser, namespace, values, option_string):
        setattr(namespace, self.const_dest, self.const)
        setattr(namespace, self.dest, values)
