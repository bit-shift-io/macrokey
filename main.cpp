/**
 *
 * Links:
 * https://cgit.freedesktop.org/~whot/evtest/tree/evtest.c
/** */


#include <stdio.h>
#include <fcntl.h>   // open
#include <stdlib.h>
#include <string.h>  // strerror
#include <errno.h>
#include <stdint.h>
#include <assert.h>
#include <unistd.h>  // daemon, close
#include <linux/input.h>
#include <iostream>
#include <vector>

#include <key_util.h>

// 0 = released, 1 = pressed, 2 repeat
#define EV_PRESSED 1
#define EV_RELEASED 0
#define EV_REPEAT 2

using namespace std;

/**
 * Exit with return code -1 if user does not have root privileges
 */
static void rootCheck() {
   if (geteuid() != 0) {
      printf("Must run as root\n");
      exit(-1);
   }
}

/**
 * Write event to device
 */
static void write_event(int fd, input_event ev) {
    ssize_t ret;
    bool single_event = true;
    ret = write(fd, &ev, sizeof(struct input_event));
    if (ret < 0) {
            perror("write");
            close(fd);
            return;
    }

    if (single_event == false) {
            ev.value ^= 1;
            ret = write(fd, &ev, sizeof(struct input_event));
            if (ret < 0) {
                   perror("write");
                   close(fd);
                   return;
            }
    }
}

/**
 * Structure for input devices
 */
class event_device
{
public:
    string device;
    bool lock;
    int fd; // filedevice

    event_device(string p_device, bool p_lock){
        device = p_device;
        lock = p_lock;
        // O_RDWR, O_RDONLY
        fd = open(device.c_str(), O_RDWR);
        if (fd == -1) {
            printf("Failed to open event device: %s.\n", device.c_str());
            exit;
        }

        // open input by name??
        //char name[256] = "Unknown";
        //memset(name, 0, sizeof(name));
        //result = ioctl(fd, EVIOCGNAME(sizeof(name)), name);
        //printf ("Reading From : %s (%s)\n", device, name);

        // lock/grab all input
        if (lock) {
            int result = 0;
            result = ioctl(fd, EVIOCGRAB, 1);
            printf("Exclusive access: %s\n", (result == 0) ? "SUCCESS" : "FAILURE");
        }
    }
};


int main(int argc, char* argv[])
{
    // check root permissions
    rootCheck();

    struct input_event ev[64]; //input event
    input_event event;
    int numevents;
    int size = sizeof(struct input_event);
    int rd;
    vector<event_device*> device_list;
    fd_set fds; // file device set
    int maxfd;
    int result = 0;

    // add devices here
    // true if we want exclusive access
    device_list.push_back( new event_device("/dev/input/event0", true) );

    // make sure we have some input
    if (device_list.size() == 0) {
        exit(1);
    }

    // main loop
    while (1)
    {
        // setup the set ready for select
        // https://linux.die.net/man/2/select
        FD_ZERO(&fds);
        maxfd = -1;
        for (int i = 0; i < device_list.size(); ++i) {
            FD_SET(device_list[i]->fd, &fds);
            if (maxfd < device_list[i]->fd) maxfd = device_list[i]->fd;
        }

        // read devices to see when it has input
        result = select(maxfd+1, &fds, NULL, NULL, NULL);
        if (result == -1) {
            break;
        }

        // output what we have to the user
        for (int i = 0; i < device_list.size(); ++i) {

            if (!FD_ISSET(device_list[i]->fd, &fds)) {
                continue;
            }

            // read from device
            if ((rd = read(device_list[i]->fd, ev, size * 64)) < size) {
                continue;
            }

            // use the events!
            numevents = rd / size;
            uint8_t shift_pressed = 0;

            for (int j = 0; j < numevents; ++j) {
                event = ev[j];

                if (event.type == EV_KEY) {
                    if (event.value == EV_PRESSED) {
                        if (isShift(event.code)) {
                            shift_pressed++;
                        }
                        char *name = getKeyText(event.code, shift_pressed);
                        if (strcmp(name, UNKNOWN_KEY) != 0) {
                            printf("key: %s\n", name);

                            // try emulate a keypress
                            input_event temp_event;
                            temp_event.type = EV_KEY;
                            temp_event.value = EV_PRESSED;
                            temp_event.code = KEY_D;
                            write_event(device_list[0]->fd,temp_event);

                            temp_event.type = EV_KEY;
                            temp_event.value = EV_RELEASED;
                            temp_event.code = KEY_D;
                            write_event(device_list[0]->fd,temp_event);
                        }
                    } else if (event.value == EV_RELEASED) {
                        if (isShift(event.code)) {
                            shift_pressed--;
                        }
                    }
                }
                assert(shift_pressed >= 0 && shift_pressed <= 2);
            }
        }
    }

    printf("Exiting.\n");

    // clean and close devices
    for (int i = 0; i < device_list.size(); ++i) {
        result = ioctl(device_list[i]->fd, EVIOCGRAB, 0);
        close(device_list[i]->fd);
    }

    return 0;
}
/*
int main(int argc, char **argv) {
   rootCheck();

   Config config;
   parseOptions(argc, argv, &config);

   int kbd_fd = openKeyboardDeviceFile(config.deviceFile);
   assert(kbd_fd > 0);

   // disable log and daemon for now
   /*
   FILE *logfile = fopen(config.logFile, "a");
   if (logfile == NULL) {
      LOG_ERROR("Could not open log file");
      exit(-1);
   }

   // We want to write to the file on every keypress, so disable buffering
   setbuf(logfile, NULL);

   // Daemonize process. Don't change working directory but redirect standard
   // inputs and outputs to /dev/null

   if (daemon(1, 0) == -1) {
      LOG_ERROR("%s", strerror(errno));
      exit(-1);
   }
    * /
   uint8_t shift_pressed = 0;
   input_event event;
   while (read(kbd_fd, &event, sizeof(input_event)) > 0) {
      if (event.type == EV_KEY) {
         if (event.value == KEY_PRESS) {
            if (isShift(event.code)) {
               shift_pressed++;
            }
            char *name = getKeyText(event.code, shift_pressed);
            if (strcmp(name, UNKNOWN_KEY) != 0) {
               LOG("%s", name);
               printf("key: %s", name);
               fputs(name, logfile);
            }
         } else if (event.value == KEY_RELEASE) {
            if (isShift(event.code)) {
               shift_pressed--;
            }
         }
      }
      assert(shift_pressed >= 0 && shift_pressed <= 2);
   }

   Config_cleanup(&config);
   fclose(logfile);
   close(kbd_fd);
   return 0;
}
*/
