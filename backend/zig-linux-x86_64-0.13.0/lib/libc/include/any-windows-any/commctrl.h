/**
 * This file is part of the mingw-w64 runtime package.
 * No warranty is given; refer to the file DISCLAIMER within this package.
 */

#include <winapifamily.h>

#ifndef _INC_COMMCTRL
#define _INC_COMMCTRL

#if WINAPI_FAMILY_PARTITION (WINAPI_PARTITION_DESKTOP)

#include <_mingw_unicode.h>

#ifndef _WINRESRC_
#ifndef _WIN32_IE
#define _WIN32_IE 0x0501
#endif
#endif

#ifndef _HRESULT_DEFINED
#define _HRESULT_DEFINED
typedef LONG HRESULT;
#endif

#ifndef NOUSER
#ifndef WINCOMMCTRLAPI
#ifndef _COMCTL32_
#define WINCOMMCTRLAPI DECLSPEC_IMPORT
#else
#define WINCOMMCTRLAPI
#endif
#endif

#ifdef __cplusplus
extern "C" {
#endif

#include <prsht.h>

#ifndef SNDMSG
#ifdef __cplusplus
#define SNDMSG ::SendMessage
#else
#define SNDMSG SendMessage
#endif
#endif

  WINCOMMCTRLAPI void WINAPI InitCommonControls(void);

  typedef struct tagINITCOMMONCONTROLSEX {
    DWORD dwSize;
    DWORD dwICC;
  } INITCOMMONCONTROLSEX,*LPINITCOMMONCONTROLSEX;

#define ICC_LISTVIEW_CLASSES 0x1
#define ICC_TREEVIEW_CLASSES 0x2
#define ICC_BAR_CLASSES 0x4
#define ICC_TAB_CLASSES 0x8
#define ICC_UPDOWN_CLASS 0x10
#define ICC_PROGRESS_CLASS 0x20
#define ICC_HOTKEY_CLASS 0x40
#define ICC_ANIMATE_CLASS 0x80
#define ICC_WIN95_CLASSES 0xff
#define ICC_DATE_CLASSES 0x100
#define ICC_USEREX_CLASSES 0x200
#define ICC_COOL_CLASSES 0x400
#define ICC_INTERNET_CLASSES 0x800
#define ICC_PAGESCROLLER_CLASS 0x1000
#define ICC_NATIVEFNTCTL_CLASS 0x2000
#define ICC_STANDARD_CLASSES 0x4000
#define ICC_LINK_CLASS 0x8000

  WINCOMMCTRLAPI WINBOOL WINAPI InitCommonControlsEx(const INITCOMMONCONTROLSEX *);

#define ODT_HEADER 100
#define ODT_TAB 101
#define ODT_LISTVIEW 102

#define LVM_FIRST 0x1000
#define TV_FIRST 0x1100
#define HDM_FIRST 0x1200
#define TCM_FIRST 0x1300
#define PGM_FIRST 0x1400
#define ECM_FIRST 0x1500
#define BCM_FIRST 0x1600
#define CBM_FIRST 0x1700

#define CCM_FIRST 0x2000
#define CCM_LAST (CCM_FIRST+0x200)
#define CCM_SETBKCOLOR (CCM_FIRST+1)
#define CCM_SETCOLORSCHEME (CCM_FIRST+2)
#define CCM_GETCOLORSCHEME (CCM_FIRST+3)
#define CCM_GETDROPTARGET (CCM_FIRST+4)
#define CCM_SETUNICODEFORMAT (CCM_FIRST+5)
#define CCM_GETUNICODEFORMAT (CCM_FIRST+6)
#define CCM_SETVERSION (CCM_FIRST+0x7)
#define CCM_GETVERSION (CCM_FIRST+0x8)
#define CCM_SETNOTIFYWINDOW (CCM_FIRST+0x9)
#define CCM_SETWINDOWTHEME (CCM_FIRST+0xb)
#define CCM_DPISCALE (CCM_FIRST+0xc)

#define COMCTL32_VERSION 6

  typedef struct tagCOLORSCHEME {
    DWORD dwSize;
    COLORREF clrBtnHighlight;
    COLORREF clrBtnShadow;
  } COLORSCHEME,*LPCOLORSCHEME;

#define INFOTIPSIZE 1024

#define HANDLE_WM_NOTIFY(hwnd,wParam,lParam,fn) (fn)((hwnd),(int)(wParam),(NMHDR *)(lParam))
#define FORWARD_WM_NOTIFY(hwnd,idFrom,pnmhdr,fn) (LRESULT)(fn)((hwnd),WM_NOTIFY,(WPARAM)(int)(idFrom),(LPARAM)(NMHDR *)(pnmhdr))

#define NM_OUTOFMEMORY (NM_FIRST-1)
#define NM_CLICK (NM_FIRST-2)
#define NM_DBLCLK (NM_FIRST-3)
#define NM_RETURN (NM_FIRST-4)
#define NM_RCLICK (NM_FIRST-5)
#define NM_RDBLCLK (NM_FIRST-6)
#define NM_SETFOCUS (NM_FIRST-7)
#define NM_KILLFOCUS (NM_FIRST-8)
#define NM_CUSTOMDRAW (NM_FIRST-12)
#define NM_HOVER (NM_FIRST-13)
#define NM_NCHITTEST (NM_FIRST-14)
#define NM_KEYDOWN (NM_FIRST-15)
#define NM_RELEASEDCAPTURE (NM_FIRST-16)
#define NM_SETCURSOR (NM_FIRST-17)
#define NM_CHAR (NM_FIRST-18)
#define NM_TOOLTIPSCREATED (NM_FIRST-19)
#define NM_LDOWN (NM_FIRST-20)
#define NM_RDOWN (NM_FIRST-21)
#define NM_THEMECHANGED (NM_FIRST-22)
#if NTDDI_VERSION >= 0x06000000
#define NM_FONTCHANGED (NM_FIRST-23)
#define NM_CUSTOMTEXT (NM_FIRST-24)
#define NM_TVSTATEIMAGECHANGING (NM_FIRST-24)
#endif

#ifndef CCSIZEOF_STRUCT
#define CCSIZEOF_STRUCT(structname,member) (((int)((LPBYTE)(&((structname*)0)->member) - ((LPBYTE)((structname*)0))))+sizeof(((structname*)0)->member))
#endif

  typedef struct tagNMTOOLTIPSCREATED {
    NMHDR hdr;
    HWND hwndToolTips;
  } NMTOOLTIPSCREATED,*LPNMTOOLTIPSCREATED;

  typedef struct tagNMMOUSE {
    NMHDR hdr;
    DWORD_PTR dwItemSpec;
    DWORD_PTR dwItemData;
    POINT pt;
    LPARAM dwHitInfo;
  } NMMOUSE,*LPNMMOUSE;

  typedef NMMOUSE NMCLICK;
  typedef LPNMMOUSE LPNMCLICK;

  typedef struct tagNMOBJECTNOTIFY {
    NMHDR hdr;
    int iItem;
#ifdef __IID_DEFINED__
    const IID *piid;
#else
    const void *piid;
#endif
    void *pObject;
    HRESULT hResult;
    DWORD dwFlags;
  } NMOBJECTNOTIFY,*LPNMOBJECTNOTIFY;

  typedef struct tagNMKEY {
    NMHDR hdr;
    UINT nVKey;
    UINT uFlags;
  } NMKEY,*LPNMKEY;

  typedef struct tagNMCHAR {
    NMHDR hdr;
    UINT ch;
    DWORD dwItemPrev;
    DWORD dwItemNext;
  } NMCHAR,*LPNMCHAR;

#if _WIN32_IE >= 0x0600
  typedef struct tagNMCUSTOMTEXT {
    NMHDR hdr;
    HDC hDC;
    LPCWSTR lpString;
    int nCount;
    LPRECT lpRect;
    UINT uFormat;
    WINBOOL fLink;
  } NMCUSTOMTEXT,*LPNMCUSTOMTEXT;
#endif

#define NM_FIRST (0U- 0U)
#define NM_LAST (0U- 99U)

#define LVN_FIRST (0U-100U)
#define LVN_LAST (0U-199U)

#define HDN_FIRST (0U-300U)
#define HDN_LAST (0U-399U)

#define TVN_FIRST (0U-400U)
#define TVN_LAST (0U-499U)

#define TTN_FIRST (0U-520U)
#define TTN_LAST (0U-549U)

#define TCN_FIRST (0U-550U)
#define TCN_LAST (0U-580U)

#ifndef CDN_FIRST
#define CDN_FIRST (0U-601U)
#define CDN_LAST (0U-699U)
#endif

#define TBN_FIRST (0U-700U)
#define TBN_LAST (0U-720U)

#define UDN_FIRST (0U-721)
#define UDN_LAST (0U-729U)
#define DTN_FIRST (0U-740U)
#define DTN_LAST (0U-745U)

#define MCN_FIRST (0U-746U)
#define MCN_LAST (0U-752U)

#define DTN_FIRST2 (0U-753U)
#define DTN_LAST2 (0U-799U)

#define CBEN_FIRST (0U-800U)
#define CBEN_LAST (0U-830U)
#define RBN_FIRST (0U-831U)
#define RBN_LAST (0U-859U)

#define IPN_FIRST (0U-860U)
#define IPN_LAST (0U-879U)
#define SBN_FIRST (0U-880U)
#define SBN_LAST (0U-899U)
#define PGN_FIRST (0U-900U)
#define PGN_LAST (0U-950U)

#ifndef WMN_FIRST
#define WMN_FIRST (0U-1000U)
#define WMN_LAST (0U-1200U)
#endif

#define BCN_FIRST (0U-1250U)
#define BCN_LAST (0U-1350U)

#if NTDDI_VERSION >= 0x06000000
#define TRBN_FIRST (0U-1501U)
#define TRBN_LAST (0U-1519U)
#endif

#define MSGF_COMMCTRL_BEGINDRAG 0x4200
#define MSGF_COMMCTRL_SIZEHEADER 0x4201
#define MSGF_COMMCTRL_DRAGSELECT 0x4202
#define MSGF_COMMCTRL_