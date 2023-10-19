# https://github.com/puzzle/lightning-beer-tap/blob/master/gpio_handler/gpio_handler.py

#!/usr/bin/env python
# -*- coding:utf-8 -*-

###################
# P26 ----> r_ch1 #
# P20 ----> r_ch2 #
# P21 ----> r_ch3 #
###################

import RPi.GPIO as GPIO
import argparse
import time

# Channels
r_ch1 = 26
r_ch2 = 20
r_ch3 = 21

# time constants in seconds
# single tap
t_beer = 1.5
t_cocktail = 1.5

t_flush = 10

# Syntax suger because of negative logic
S_ON = GPIO.LOW
S_OFF = GPIO.HIGH

def cli_args_parser():
    """
        Argument parser configuration
    """
    parser = argparse.ArgumentParser(
            description='Bitcoin Bay Bartender cli',
            formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )

    parser.add_argument(
            '-p',
            '--products',
            action='store',
            dest='products',
            help="Drink choice [beer, cocktail]"
    )

    parser.add_argument(
            '-t',
            '--test',
            action='store_true',
            help="Test mode which tests all available channels"
    )

    parser.add_argument(
            '-f',
            '--flush',
            action='store_true',
            help="Flush tap for 10s!"
    )

    return parser.parse_args()

def __setup_GPIO(channel=r_ch1):
    """
        Setup all GPIOs, set output mode, and set gpio mode to bcm
    """
    GPIO.setwarnings(False)
    GPIO.setmode(GPIO.BCM)

    GPIO.setup(channel, GPIO.OUT)
    __set_gpio(channel, S_OFF)

def __set_gpio(channel=r_ch1, value=S_OFF):
    """
        Try to safely change the value of a gpio, catch exception if it fails
        TODO: Exception Handling
    """
    try:
        GPIO.output(channel, value)
    except:
        print("GPIO Error")
        GPIO.cleanup()

def gpio_test():
    """
        Test all channels
    """
    for i, gpio in enumerate([r_ch1, r_ch2, r_ch3], start=1):
        # Setup gpio pin
        __setup_GPIO(gpio)

        __set_gpio(gpio, S_ON)
        time.sleep(0.1)
        __set_gpio(gpio, S_OFF)
        time.sleep(0.1)

def draw_beer(channel=r_ch1, wait=t_beer):
    """
        Draw a delicious beer, keep the tap on for n_wait seconds
    """
    # Setup gpio pin
    __setup_GPIO(channel)

    __set_gpio(channel, S_ON)
    time.sleep(wait)
    __set_gpio(channel, S_OFF)

if __name__ == "__main__":
    """
        Main if not loaded as module
    """
    # parse arguments
    args = cli_args_parser()

    # call functions according to the given arguments
    if args.test:
        print("Test mode enabled")
        gpio_test()
    elif args.flush:
        print("Choice: Flush tap")
        draw_beer(r_ch1, t_flush)
    elif args.products == "beer":
        print("Choice: Beer")
        draw_beer(r_ch1, t_beer)
    elif args.products == "cocktail":
        print("Choice: Cocktail")
        draw_beer(r_ch1, t_cocktail)
    else:
        print("sumting wong")
