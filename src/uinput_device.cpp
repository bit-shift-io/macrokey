/**
 * Emulate a uinput device
 * UHID is a pain!
 * 
 * Test inputs using evdevtest linux app
 * 
 * https://stackoverflow.com/questions/23092855/how-to-generate-keyboard-input-using-libevdev-in-c
 */


#include "uinput_device.h"
#include <linux/uinput.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

int emit(int fd, int type, int code, int val)
{
    //printf("emit:    fd: %i      type: %i      code: %i    val: %i\n", fd, type, code, val);

   struct input_event ie;

   ie.type = type;
   ie.code = code;
   ie.value = val;
   /* timestamp values below are ignored */
   ie.time.tv_sec = 0;
   ie.time.tv_usec = 0;

   ssize_t ret = write(fd, &ie, sizeof(ie));
   if (ret < 0) {
        fprintf(stderr, "Cannot write to uinput: %m\n");
        return ret;
    } 
        
    return 0;
}

uinput_device::uinput_device() {
   fd = -1; 
}

uinput_device::~uinput_device() {
    ioctl(fd, UI_DEV_DESTROY);
    close(fd);
    fd = -1;
}

void uinput_device::open() {
    struct uinput_setup usetup;

   fd = ::open("/dev/uinput", O_WRONLY | O_NONBLOCK);

   /*
    * The ioctls below will enable the device that is about to be
    * created, to pass key events, in this case the all keys (example shows space key).
    */
   ioctl(fd, UI_SET_EVBIT, EV_KEY);

   for (int i = 1; i < 255; ++i) {
       ioctl(fd, UI_SET_KEYBIT, i /*eg. KEY_SPACE*/);
   }

    /*
    mouse buttons
    */
   ioctl(fd, UI_SET_KEYBIT, BTN_LEFT);
   ioctl(fd, UI_SET_KEYBIT, BTN_RIGHT);
   ioctl(fd, UI_SET_KEYBIT, BTN_MIDDLE);
   // TODO: add extra mouse button support

   memset(&usetup, 0, sizeof(usetup));
   usetup.id.bustype = BUS_USB;
   usetup.id.vendor = 0x1234; /* sample vendor */
   usetup.id.product = 0x5678; /* sample product */
   strcpy(usetup.name, "macrokey virtual mouse & keyboard");

   ioctl(fd, UI_DEV_SETUP, &usetup);
   ioctl(fd, UI_DEV_CREATE);

   /*
    * On UI_DEV_CREATE the kernel will create the device node for this
    * device. We are inserting a pause here so that userspace has time
    * to detect, initialize the new device, and can start listening to
    * the event, otherwise it will not notice the event we are about
    * to send. This pause is only needed in our example code!
    */
   sleep(1);

   printf("Macrokey: uinput device opened with fd: %i\n", fd);
}

int uinput_device::send_event(int key, int state) {
/*
    switch (key) {
    case BTN_LEFT:
    case BTN_RIGHT:
    case BTN_MIDDLE:
        printf("mouse event!\n");
        break;
    }
*/

    emit(fd, EV_KEY, key /*eg. KEY_SPACE*/, state /* eg. EV_PRESSED */);
    // send sync report
    return emit(fd, EV_SYN, SYN_REPORT, 0);
}