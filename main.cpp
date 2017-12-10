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
#include <map>
#include <dirent.h>

#include <uhid_device.h>

using namespace std;

#define ID_FOOTSWITCH 0


/**
 * Structure for input devices
 */
class event_device
{
public:
    string device;
    string name;
    int id;

    bool lock;
    int fd; // filedevice

    event_device(string p_device) {
        fd = -1;
        id = -1;
        device = p_device;

        // read name
        char nm[256] = "???";
        fd = ::open(device.c_str(), O_RDONLY);
        if (fd < 0)
            return;
        ioctl(fd, EVIOCGNAME(sizeof(nm)), nm);

        name = nm;
        fprintf(stderr, "Device: %s, %s\n", device.c_str(), name.c_str());

        ::close(fd);
    }

    void open(bool p_lock){
        fprintf(stderr, "Opening Device: %s, %s\n", device.c_str(), name.c_str());

        lock = p_lock;
        // O_RDWR, O_RDONLY
        fd = ::open(device.c_str(), O_RDWR);
        if (fd == -1) {
            fprintf(stderr, "Failed to open event device: %s.\n", device.c_str());
            exit;
        }

        // lock/grab all input
        if (lock) {
            int result = 0;
            result = ioctl(fd, EVIOCGRAB, 1);
            fprintf(stderr, "Exclusive access: %s\n", (result == 0) ? "SUCCESS" : "FAILURE");
        }
    }

    void close() {
        int result = ioctl(fd, EVIOCGRAB, 0);
        ::close(fd);
    }

    bool is_open() { return fd != -1; }
};



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
static vector<event_device*> get_devices() {

    vector<event_device*> list;

    struct dirent **namelist;
    int i, ndev, devnum;
    char *filename;

    ndev = scandir("/dev/input", &namelist, is_event_device, alphasort);
    if (ndev <= 0)
        return list;

    fprintf(stderr, "Available devices:\n");
    // list device and names
    for (i = 0; i < ndev; i++)
    {
        char fname[64];
        int fd = -1;
        char name[256] = "???";

        snprintf(fname, sizeof(fname),
             "%s/%s", "/dev/input", namelist[i]->d_name);

        list.push_back(new event_device(fname));
    }

    return list;
}


/**
 * process input event
 */
void process_input(event_device *device, input_event *event, uhid_device* virtual_device){
    if (device->id != ID_FOOTSWITCH) {
        return;
    }

    if (event->type == EV_KEY) {
        /*
        if (event->value == EV_PRESSED)
            fprintf(stderr, "EV_PRESSED\n");

        if (event->value == EV_RELEASED)
            fprintf(stderr, "EV_RELEASED\n");
        */
        if (event->code == KEY_A) {
            virtual_device->send_event(BTN_RIGHT, event->value);
        }

        if (event->code == KEY_B) {
            virtual_device->send_event(BTN_MIDDLE, event->value);
        }

        if (event->code == KEY_C) {
            virtual_device->send_event(BTN_LEFT, event->value);
        }
    }
}

int main(int argc, char* argv[])
{
    // check root permissions
    root_check();

    // list input devices
    vector<event_device*> all_device_list = get_devices();
    vector<event_device*> device_list;

    // open appropriate devices
    for (int i = 0; i < all_device_list.size(); ++i) {
        if (all_device_list[i]->name.find("FootSwitch") != string::npos) {
            all_device_list[i]->open(true);
            all_device_list[i]->id = ID_FOOTSWITCH;
            device_list.push_back(all_device_list[i]);
        }
    }

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

    // make sure we have some input
    if (device_list.size() == 0) {
        printf("Please configure at least 1 device\n");
        exit(1);
    }

    // create a virtual uhid device
    uhid_device* virtual_device = new uhid_device();

    // main loop
    while (1)
    {
        // setup the set ready for select
        // https://linux.die.net/man/2/select
        FD_ZERO(&fds);
        maxfd = -1;
        for (int i = 0; i < device_list.size(); ++i) {
            FD_SET(device_list[i]->fd, &fds);
            if (maxfd < device_list[i]->fd)
                maxfd = device_list[i]->fd;
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
                process_input(device_list[i], &ev[j], virtual_device);
            }
        }
    }

    // clean and close devices
    for (int i = 0; i < device_list.size(); ++i) {
        device_list[i]->close();
    }

    delete virtual_device;

    return 0;
}
