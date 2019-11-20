/**
 * macrokey
 * a small utility to emulate key presses and monitor input devices
 * on linux
 *
/** */

//#include <map>
//#include <fcntl.h>   // open
//#include <string.h>  // strerror
//#include <errno.h>
//#include <assert.h>
//#include <unistd.h>  // daemon, close
//#include <linux/uhid.h>
//#include <stdio.h>
//#include <stdlib.h>
//#include <stdint.h>
//#include <linux/input.h>
//#include <iostream>
//#include <vector>
#include <dirent.h>
#include <boost/python.hpp>
#include "src/event_device.h"
#include "src/uhid_device.h"


using namespace std;

//#define ID_FOOTSWITCH 0
uhid_device* virtual_device = NULL; // the app create its own virtual device for playback/emulation of events
vector<event_device*> system_device_list; // keep a list of system devices
vector<event_device*> device_list; // keep a list of active devices
// python callback: https://stackoverflow.com/questions/7204664/pass-callback-from-python-to-c-using-boostpython
// this is the variable that will hold a reference to the python function
PyObject *py_callback;


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
static vector<event_device*> get_system_devices() {
    vector<event_device*> list;

    struct dirent **namelist;
    int i, ndev, devnum;
    char *filename;

    ndev = scandir("/dev/input", &namelist, is_event_device, alphasort);
    if (ndev <= 0)
        return list;

    fprintf(stderr, "System devices:\n");
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


void initialize() {
    // check root permissions
    root_check();

    // list input devices
    system_device_list = get_system_devices();
}

/**
 * the following function will invoked from python to populate the call back reference
 */
PyObject *set_py_callback(PyObject *callable)
{
    initialize();
    py_callback = callable;       /* Remember new callback */
    return Py_None;
}


/**
 * process input event
 */
void process_input(event_device *device, input_event *event, uhid_device* virtual_device){
    /*
    if (device->id != ID_FOOTSWITCH) {
        return;
    }*/
    
    // invoke the python function
    boost::python::call<void>(py_callback, event->type, event->code, event->value);
}


void send_event_to_virtual_device(int key, int state) {
    virtual_device->send_event(key, state);
}

/**
 * open a device for event reading
 **/
void open_device(string p_device_name, bool p_exclusive_lock) {
    const char *device_name = p_device_name.c_str();
    bool found = false;

    // loop system devices
    for (int i = 0; i < system_device_list.size(); ++i) {

        // append device if matches
        if (system_device_list[i]->name.find(device_name) != string::npos) {
            printf("Device match: %s, %s\n", device_name, system_device_list[i]->name);
            system_device_list[i]->open(p_exclusive_lock);
            //system_device_list[i]->id = ID_FOOTSWITCH;
            device_list.push_back(system_device_list[i]);
            found = true;
        }     
    }

    if (!found) {
        printf("Device match: none, %s\n", device_name);
    }
}


void run() {
    // make sure we have some input
    if (device_list.size() == 0) {
        printf("Please configure at least 1 device\n");
        exit(1);
    }

    // create a virtual uhid device
    virtual_device = new uhid_device();

    // define some variables
    struct input_event ev[64]; //input event
    //input_event event;
    int numevents;
    int size = sizeof(struct input_event);
    int rd;
    fd_set fds; // file device set
    int maxfd;
    int result = 0;

    printf("Running main loop...\n");

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
        timeval timeout;
        timeout.tv_sec = 0;
        timeout.tv_usec = 100000; // 100 milliseconds
        result = select(maxfd+1, &fds, NULL, NULL, &timeout);
        if (result == -1) {
            break;
        }
        
        // Ensure that the current thread is ready to call the Python C API 
        PyGILState_STATE state = PyGILState_Ensure();

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
        
        // send a dummy event, just to keep the python timers ticking over
        boost::python::call<void>(py_callback, 0, 0, 0);
            
        // release the global interpreter lock so other threads can resume execution
        PyGILState_Release(state);
    }

    // remove virtual device
    delete virtual_device;

    // clean and close devices
    for (int i = 0; i < device_list.size(); ++i) {
        device_list[i]->close();
    }
}


int main(int argc, char* argv[])
{
    // initialize devices etc..
    initialize();
    run();
    return 0;
}

BOOST_PYTHON_MODULE(macrokey)
{
    using namespace boost::python;
    def("send_event_to_virtual_device", send_event_to_virtual_device);
    def("set_py_callback", set_py_callback);
    def("open_device", open_device);
    def("run", run);
}
