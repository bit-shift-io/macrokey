/**
 * macrokey
 * a small utility to emulate key presses and monitor input devices
 * on linux
 *
 */

#include <dirent.h>
#include <boost/python.hpp>
#include "src/event_device.h"
#include "src/uinput_device.h"

using namespace std;

uinput_device* virtual_device = NULL; // the app create its own virtual device for playback/emulation of events
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

        snprintf(fname, sizeof(fname), "%s/%s", "/dev/input", namelist[i]->d_name);

        list.push_back(new event_device(fname));
    }

    printf("\n");

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
void process_input(event_device *device, input_event *event){
    /*
    // EV_MSC is keybord scancodes (we want these!)
    // EV_KEY is linux keycodes (we don't want these! or anything else such as EV_SYN)
    if (event->type != EV_MSC) {
        return;
    }*/

    //printf("c++ event:     type: %i      code: %i      value: %i    devId: %i\n", event->type, event->code, event->value, device->id);

    // invoke the python function
    if (py_callback) {
        boost::python::call<void>(py_callback, event->type, event->code, event->value, device->id);
    }
}


void send_event_to_virtual_device(int key, int state) {
    virtual_device->send_event(key, state);
}

/**
 * open a device for event reading
 **/
int open_device(string p_device_name, bool p_exclusive_lock) {
    const char *device_name = p_device_name.c_str();

    // loop system devices
    for (int i = 0; i < system_device_list.size(); ++i) {

        // append device if matches - either by name or by device
        if (system_device_list[i]->name.find(device_name) != string::npos ||
            system_device_list[i]->device.find(device_name) != string::npos) {
            printf("Device match: %s, %s, %s\n", device_name, system_device_list[i]->device.c_str(), system_device_list[i]->name.c_str());
            system_device_list[i]->open(p_exclusive_lock);
            device_list.push_back(system_device_list[i]);
            return system_device_list[i]->id;
        }     
    }

    printf("Device match: none, %s\n", device_name);
    return -1;
}

void run() {
    // make sure we have some input
    if (device_list.size() == 0) {
        printf("Please configure at least 1 device\n");
        exit(1);
    }

    // create a virtual uhid device
    virtual_device = new uinput_device();
    virtual_device->open();

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
        PyGILState_STATE state;
        if (py_callback) {
            state = PyGILState_Ensure();
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
                process_input(device_list[i], &ev[j]);
            }
        }
        
        if (py_callback) {
            // send a dummy event, just to keep the python timers ticking over
            boost::python::call<void>(py_callback, 0, 0, 0, -1);
                    
            // release the global interpreter lock so other threads can resume execution
            PyGILState_Release(state);
        }
    }
}

void done() {
    // remove virtual device
    delete virtual_device;

    // clean and close devices
    for (int i = 0; i < device_list.size(); ++i) {
        device_list[i]->close();
    }
}


int main(int argc, char* argv[])
{
    // only in here on a debug build...
    // initialize devices etc..
    return 0;
}

BOOST_PYTHON_MODULE(macrokey)
{
    using namespace boost::python;
    def("send_event_to_virtual_device", send_event_to_virtual_device);
    def("set_py_callback", set_py_callback);
    def("open_device", open_device);
    def("run", run);
    def("done", done);
}
