/**
 *
 * Links:
 * https://cgit.freedesktop.org/~whot/evtest/tree/evtest.c
 * https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/hid/uhid.txt?id=refs/tags/v4.10-rc3
 * https://github.com/torvalds/linux/blob/master/samples/uhid/uhid-example.c
 *
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
#include <linux/uhid.h>
#include <iostream>
#include <vector>
#include <dirent.h>

#include <uhid_device.h>

// 0 = released, 1 = pressed, 2 repeat
#define EV_PRESSED 1
#define EV_RELEASED 0
#define EV_REPEAT 2

using namespace std;

/**
 * Exit with return code -1 if user does not have root privileges
 */
static void root_check() {
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
 * Filter for the AutoDevProbe scandir on /dev/input.
 *
 * @param dir The current directory entry provided by scandir.
 *
 * @return Non-zero if the given directory entry starts with "event", or zero
 * otherwise.
 */
static int is_event_device(const struct dirent *dir) {
    return strncmp("event", dir->d_name, 5) == 0;
}

/**
 * Scans all /dev/input/event*, display them.
 *
 */
static void list_devices() {
    struct dirent **namelist;
    int i, ndev, devnum;
    char *filename;

    ndev = scandir("/dev/input", &namelist, is_event_device, alphasort);
    if (ndev <= 0)
        return;

    fprintf(stderr, "Available devices:\n");
    // list device and names
    for (i = 0; i < ndev; i++)
    {
        char fname[64];
        int fd = -1;
        char name[256] = "???";

        snprintf(fname, sizeof(fname),
             "%s/%s", "/dev/input", namelist[i]->d_name);
        fd = open(fname, O_RDONLY);
        if (fd < 0)
            continue;
        ioctl(fd, EVIOCGNAME(sizeof(name)), name);

        fprintf(stderr, "%s:	%s\n", fname, name);
        close(fd);
        free(namelist[i]);
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


/*
 * process input event
 */
void process_input(event_device *device, input_event *event){
    printf("event \n");
    if (event->type == EV_KEY) {
        if (event->value == EV_PRESSED) {
            if (event->code == KEY_A) {
                printf("peace\n");
            }

            /*if (isShift(event.code)) {
                shift_pressed++;
            }*/
            /*
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
            */
        } else if (event->value == EV_RELEASED) {
            /*if (isShift(event.code)) {
                shift_pressed--;
            }*/
        }
    }
    //assert(shift_pressed >= 0 && shift_pressed <= 2);
}

int main(int argc, char* argv[])
{
    // check root permissions
    root_check();

    // list input devices
    list_devices();

    // Daemonize process. Don't change working directory but redirect standard
    // inputs and outputs to /dev/null
    /*
    if (daemon(1, 0) == -1) {
       printf("%s\n", strerror(errno));
       exit(-1);
    }
    */

    struct input_event ev[64]; //input event
    //input_event event;
    int numevents;
    int size = sizeof(struct input_event);
    int rd;
    fd_set fds; // file device set
    int maxfd;
    int result = 0;

    // add devices here
    // true if we want exclusive access
    vector<event_device*> device_list;
    device_list.push_back( new event_device("/dev/input/event0", true) ); // Footswitch
    //device_list.push_back( new event_device("/dev/input/event19", false) ); // Leveno Keyboard
    //device_list.push_back( new event_device("/dev/input/event20", false) ); // Leveno Keyboard
    //device_list.push_back( new event_device("/dev/input/event5", false) ); // Gaming Mouse
    //device_list.push_back( new event_device("/dev/input/event6", false) ); // Gaming Mouse

    // make sure we have some input
    if (device_list.size() == 0) {
        printf("Please configure at least 1 device\n");
        exit(1);
    }

    // create a virtual uhid device
    uhid_device* virtual_device = new uhid_device();


    // try emulate a keypress on virtual device!
    uhid_event uev;
    uev.type = UHID_INPUT;
    uev.u.input.size = 5;

    uev.u.input.data[0] = 0x1;
    //if (btn1_down)
        uev.u.input.data[1] |= 0x1; //mouse button
    //if (btn2_down)
    //    uev.u.input.data[1] |= 0x2;
    //if (btn3_down)
     //   uev.u.input.data[1] |= 0x4;

    uev.u.input.data[2] = -20; // mouse pos x
    uev.u.input.data[3] = 20; // mouse pos u
    uev.u.input.data[4] = 0; // mouse wheel
    virtual_device->send_event(&uev);

    /*
    temp_event.type = EV_KEY;
    temp_event.value = EV_PRESSED;
    temp_event.code = KEY_D;
    virtual_device->send_event(&ev);

    temp_event.type = EV_KEY;
    temp_event.value = EV_RELEASED;
    temp_event.code = KEY_D;
    virtual_device->send_event(&ev);
*/


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
            for (int j = 0; j < numevents; ++j) {
                printf("key\n");
                process_input(device_list[i], &ev[j]);
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
